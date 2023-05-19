use {
    camino::Utf8PathBuf,
    clap::{Parser, ValueHint},
    mocha_ident::spec::{PackageIdent, RepositoryIdent},
};

/// Mocha's package manager.
#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
#[command(verbatim_doc_comment)]
pub enum Args {
    /// Install or update packages.
    ///
    /// Milk has built-in cross-compilation support,
    /// specify the target with an at-symbol, then the target.
    ///
    ///   milk add package@gnu
    ///   milk add package@musl-dynamic
    #[command(arg_required_else_help = true)]
    #[command(verbatim_doc_comment)]
    Add {
        /// Set of packages.
        #[arg(required = true)]
        #[arg(value_name = "PACKAGE")]
        packages: Vec<PackageIdent>,
    },

    /// Format package specifications.
    #[command(arg_required_else_help = true)]
    #[command(verbatim_doc_comment)]
    Fmt {
        /// Set of path specifications.
        #[arg(required = true)]
        #[arg(value_hint = ValueHint::FilePath)]
        #[arg(value_name = "SPEC")]
        specs: Vec<Utf8PathBuf>,
    },

    /// Synchronize package repositories.
    #[command(arg_required_else_help = true)]
    #[command(verbatim_doc_comment)]
    Sync {
        /// Set of repositories.
        #[arg(required = true)]
        #[arg(value_name = "REPO")]
        repositories: Vec<RepositoryIdent>,
    },
}

impl Args {
    #[inline]
    pub fn parse() -> Self {
        <Args as Parser>::parse()
    }
}
