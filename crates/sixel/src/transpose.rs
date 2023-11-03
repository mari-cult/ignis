use core::ops::Range;

#[derive(Clone, Debug)]
pub struct Transpose<'a, T> {
    columns: Range<usize>,
    rows: Range<usize>,
    slice: &'a [T],
}

impl<'a, T> Iterator for Transpose<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(column) = self.columns.next() else {
            self.rows.next()?;
            self.columns.start = 0;

            return self.next();
        };

        let index = self.rows.start + column * self.columns.end;

        Some(&self.slice[index])
    }
}

pub trait TransposeSlice<T> {
    fn transpose(&self, columns: usize, rows: usize) -> Transpose<'_, T>;
}

impl<T> TransposeSlice<T> for [T] {
    fn transpose(&self, columns: usize, rows: usize) -> Transpose<'_, T> {
        Transpose {
            columns: 0..columns,
            rows: 0..rows,
            slice: self,
        }
    }
}
