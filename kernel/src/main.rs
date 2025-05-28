#![no_main]
#![no_std]

extern crate alloc;

// Use alloc::string::String for owned strings and VecDeque for line buffer
use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec; // For Vec<(&str, Attrs)>

use core::arch::asm;
use core::{iter, ptr};
// Import core::fmt::Write for the trait implementation
use core::fmt::{self, Write};

use core_maths::CoreFloat;
use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache};
use limine::memory_map::EntryType;
use linked_list_allocator::LockedHeap;
use spin::{Mutex, Once};

mod boot;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Logger using the new println macro
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            // Use the global println! macro
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

// Framebuffer structure (as before)
struct Framebuffer {
    addr: *mut u8,
    pitch: u64,
    width: u64,
    height: u64,
    // We'll assume 32 bpp, XRGB format as common with Limine
}

unsafe impl Send for Framebuffer {}
unsafe impl Sync for Framebuffer {}

// Maximum number of lines to keep in the scrollback buffer
const MAX_CONSOLE_LOGICAL_LINES: usize = 200;
// Default font size and line height for the console
const CONSOLE_FONT_SIZE: f32 = 16.0;
const CONSOLE_LINE_HEIGHT: f32 = 18.0;

struct Console {
    framebuffer: Framebuffer,
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_buffer: Buffer,             // For rendering visible lines
    logical_lines: VecDeque<String>, // Stores all lines, including scrollback
    default_attrs: Attrs<'static>,
    font_metrics: Metrics,
    max_visible_lines: usize,
}

static CONSOLE: Once<Mutex<Console>> = Once::new();

impl Console {
    pub fn new(framebuffer: Framebuffer) -> Self {
        let mut font_system = FontSystem::new_with_fonts(iter::once(
            cosmic_text::fontdb::Source::Binary(Arc::from(include_bytes!(
                "../../assets/fonts/RobotoMono-SemiBold.ttf" // Ensure this path is correct
            ))),
        ));

        let swash_cache = SwashCache::new();
        let font_metrics = Metrics::new(CONSOLE_FONT_SIZE, CONSOLE_LINE_HEIGHT);
        let mut text_buffer = Buffer::new(&mut font_system, font_metrics);

        // Set the layout size of the cosmic_text buffer to the framebuffer dimensions
        text_buffer.set_size(
            &mut font_system,
            Some(framebuffer.width as f32),
            Some(framebuffer.height as f32),
        );

        // Calculate how many lines can be visible
        let mut max_visible_lines =
            (framebuffer.height as f32 / font_metrics.line_height).floor() as usize;
        if max_visible_lines == 0 {
            max_visible_lines = 1; // Ensure at least one line can be shown
        }

        let default_attrs = Attrs::new().color(Color::rgb(0xFF, 0xFF, 0xFF)); // Default white text

        let mut logical_lines = VecDeque::with_capacity(MAX_CONSOLE_LOGICAL_LINES);
        logical_lines.push_back(String::new()); // Start with one empty line

        Self {
            framebuffer,
            font_system,
            swash_cache,
            text_buffer,
            logical_lines,
            default_attrs,
            font_metrics,
            max_visible_lines,
        }
    }

    // Clears the framebuffer to black
    fn clear_framebuffer(&mut self) {
        let screen_size_bytes = self.framebuffer.pitch * self.framebuffer.height;
        unsafe {
            ptr::write_bytes(self.framebuffer.addr, 0x00, screen_size_bytes as usize);
        }
    }

    /// Sets the default color for text printed to the console.
    pub fn set_default_color(&mut self, color: Color) {
        self.default_attrs = Attrs::new().color(color);
    }

    /// Resets the default color to white.
    pub fn reset_default_color(&mut self) {
        self.default_attrs = Attrs::new().color(Color::rgb(0xFF, 0xFF, 0xFF));
    }

    // Renders the current visible lines to the framebuffer
    pub fn flush_and_redraw(&mut self) {
        self.clear_framebuffer();

        // Determine the slice of logical_lines to display
        let display_line_count = self.logical_lines.len().min(self.max_visible_lines);
        let start_index = self.logical_lines.len() - display_line_count;

        let mut text_spans: Vec<(&str, Attrs)> = Vec::new();

        for i in 0..display_line_count {
            let line_index_in_deque = start_index + i;
            if let Some(line_str) = self.logical_lines.get(line_index_in_deque) {
                text_spans.push((line_str.as_str(), self.default_attrs.clone()));
                // Add a newline for all but the conceptual "last line" being fed to set_rich_text,
                // if there are more lines to come or if it's not the very last line of all logical lines.
                // cosmic-text handles wrapping, so we primarily add \n to separate distinct logical lines.
                if i < display_line_count - 1 {
                    // If not the last line being pushed to spans
                    text_spans.push(("\n", self.default_attrs.clone()));
                }
            }
        }

        // If there are no lines to display (e.g., after clearing everything),
        // provide an empty span to prevent panic in set_rich_text.
        if text_spans.is_empty() {
            text_spans.push(("", self.default_attrs.clone()));
        }

        self.text_buffer.set_rich_text(
            &mut self.font_system,
            text_spans,
            &self.default_attrs, // Base attributes for the buffer
            Shaping::Advanced,
            None, // metadata_map
        );

        // Prepare framebuffer details for the drawing closure
        let fb_addr = self.framebuffer.addr;
        let fb_pitch = self.framebuffer.pitch;
        let fb_width = self.framebuffer.width;
        let fb_height = self.framebuffer.height;

        // Drawing closure - captures framebuffer details
        let drawing_closure = |x_px: i32, y_px: i32, w_px: u32, h_px: u32, color: Color| {
            // Limine's typical framebuffer format is XRGB8888 (32-bit).
            // R is at bits 16-23, G at 8-15, B at 0-7.
            let pixel_val_rgb =
                ((color.r() as u32) << 16) | ((color.g() as u32) << 8) | (color.b() as u32);

            for ry in 0..h_px {
                // Iterate over each pixel of the glyph
                for rx in 0..w_px {
                    let screen_x = x_px + rx as i32;
                    let screen_y = y_px + ry as i32;

                    if screen_x < 0
                        || screen_x >= fb_width as i32
                        || screen_y < 0
                        || screen_y >= fb_height as i32
                    {
                        continue; // Skip pixels outside framebuffer bounds
                    }

                    let offset = (screen_y as u64 * fb_pitch) + (screen_x as u64 * 4); // 4 bytes per pixel
                    let pixel_ptr = unsafe { fb_addr.add(offset as usize).cast::<u32>() };

                    if color.a() == 255 {
                        // Opaque glyph pixel
                        unsafe { pixel_ptr.write_volatile(pixel_val_rgb) };
                    } else if color.a() > 0 {
                        // Transparent glyph pixel, blend with background
                        let bg_pixel_val = unsafe { pixel_ptr.read_volatile() };

                        // Extract RGB from background (XRGB format)
                        let bg_b = (bg_pixel_val & 0x0000FF) as u8;
                        let bg_g = ((bg_pixel_val & 0x00FF00) >> 8) as u8;
                        let bg_r = ((bg_pixel_val & 0xFF0000) >> 16) as u8;

                        let fg_r = color.r();
                        let fg_g = color.g();
                        let fg_b = color.b();
                        let alpha_norm = color.a() as f32 / 255.0;

                        // Standard alpha blending: C_out = C_fg * A_fg + C_bg * (1 - A_fg)
                        let out_r =
                            (fg_r as f32 * alpha_norm + bg_r as f32 * (1.0 - alpha_norm)) as u8;
                        let out_g =
                            (fg_g as f32 * alpha_norm + bg_g as f32 * (1.0 - alpha_norm)) as u8;
                        let out_b =
                            (fg_b as f32 * alpha_norm + bg_b as f32 * (1.0 - alpha_norm)) as u8;

                        let blended_pixel_val_rgb =
                            ((out_r as u32) << 16) | ((out_g as u32) << 8) | (out_b as u32);
                        unsafe { pixel_ptr.write_volatile(blended_pixel_val_rgb) };
                    }
                    // If color.a() == 0, glyph pixel is fully transparent, so do nothing.
                }
            }
        };

        self.text_buffer.draw(
            &mut self.font_system,
            &mut self.swash_cache,
            Color::rgba(0, 0, 0, 0), // Transparent background for text layout areas
            drawing_closure,
        );
    }
}

// Implement core::fmt::Write for our Console
impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            if ch == '\n' {
                // If current line is full or we explicitly hit newline, add a new line
                if self.logical_lines.len() >= MAX_CONSOLE_LOGICAL_LINES {
                    self.logical_lines.pop_front(); // Maintain scrollback limit
                }
                self.logical_lines.push_back(String::new());
            } else if ch.is_control() {
                // Handle other control characters if needed (e.g., backspace, tabs)
                // For now, ignore them or print a placeholder
                if ch == '\r' { /* ignore carriage return for now, or handle as newline */ }
                // else { self.logical_lines.back_mut().unwrap().push('?'); } // Placeholder for unhandled control chars
            } else {
                // Add character to the current line
                if self.logical_lines.is_empty() {
                    // Should not happen if initialized with one line
                    self.logical_lines.push_back(String::new());
                }
                self.logical_lines.back_mut().unwrap().push(ch);
            }
        }
        Ok(())
    }
}

// Print macros
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Helper function for print macros to lock the console and write
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    if let Some(console_mutex) = CONSOLE.get() {
        let mut console_guard = console_mutex.lock();
        // The Write trait takes care of appending to logical_lines
        console_guard.write_fmt(args).unwrap();
        // Redraw the console after the print operation
        console_guard.flush_and_redraw();
    }
    // If console is not initialized, output might be lost (e.g. very early panic)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmain() -> ! {
    // Initialize memory allocator
    let memory_map_response = boot::MEMORY_MAP_REQUEST
        .get_response()
        .expect("Failed to get memory map");

    let (base, len) = memory_map_response
        .entries()
        .iter()
        .filter(|entry| entry.entry_type == EntryType::USABLE)
        .max_by_key(|entry| entry.length)
        .map(|entry| (entry.base, entry.length))
        .expect("No usable memory region found");

    let base_ptr = ptr::with_exposed_provenance_mut(base as usize);
    unsafe {
        ALLOCATOR.lock().init(base_ptr, len as usize);
    }

    // Initialize Framebuffer
    let framebuffer_response = boot::FRAMEBUFFER_REQUEST
        .get_response()
        .expect("Failed to get framebuffer response");
    let limine_fb = framebuffer_response
        .framebuffers()
        .next()
        .expect("No framebuffer available");

    let kernel_framebuffer = Framebuffer {
        addr: limine_fb.addr(),
        pitch: limine_fb.pitch(),
        width: limine_fb.width(),
        height: limine_fb.height(),
    };

    // Initialize Console
    CONSOLE.call_once(|| Mutex::new(Console::new(kernel_framebuffer)));

    // Initialize Logger
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .unwrap();

    // Test printing
    println!("Hello from the kernel!");
    println!(
        "This is line 2. Framebuffer: {}x{}",
        limine_fb.width(),
        limine_fb.height()
    );
    log::info!("This is an info log message.");

    let mut counter = 0;
    loop {
        println!("Counter: {}", counter);
        counter += 1;
        // Simple delay loop (very basic, not for production)
        for _ in 0..10_000_000 {
            unsafe {
                asm!("nop");
            }
        }
        if counter > 5 {
            // Print a few lines then panic
            break;
        }
    }

    panic!("Kernel main finished (intentional panic).");

    // Loop forever (hlt) - panic above will prevent reaching here directly
    // loop {
    //     unsafe {
    //         asm!("hlt");
    //     }
    // }
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    // Attempt to print panic info to console, if CONSOLE is initialized
    if CONSOLE.get().is_some() {
        // Change color to red for panic message
        if let Some(console_mutex) = CONSOLE.get() {
            let mut console_guard = console_mutex.lock();
            console_guard.set_default_color(Color::rgb(0xFF, 0x20, 0x20)); // Red
        }

        println!("\n--- KERNEL PANIC ---");
        println!("{info}");

        // Reset color if desired, though system is halting
        if let Some(console_mutex) = CONSOLE.get() {
            let mut console_guard = console_mutex.lock();
            console_guard.reset_default_color();
        }
    } else {
        // Fallback if console is not available (e.g., very early panic)
        // One could try writing to serial port here if available.
    }

    loop {
        unsafe {
            asm!("cli", "hlt");
        }
    }
}
