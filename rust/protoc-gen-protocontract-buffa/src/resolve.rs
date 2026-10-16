//! Rust paths of the message types a method takes and returns.

use buffa_codegen::CodeGenConfig;
use buffa_codegen::context::CodeGenContext;
use buffa_codegen::generated::descriptor::FileDescriptorProto;
use proc_macro2::TokenStream;

use crate::Error;
use crate::paths::{is_absolute, path_tokens};

/// Resolves proto type names to the Rust paths `protoc-gen-buffa` gives
/// them, through buffa's own type map: `buffa_module` and `extern_path`
/// apply with the longest prefix winning, and well-known types map to
/// `::buffa_types::google::protobuf` unless a mapping covers them.
pub(crate) struct TypeResolver<'a> {
    context: CodeGenContext<'a>,
}

impl<'a> TypeResolver<'a> {
    pub(crate) fn new(
        files: &'a [FileDescriptorProto],
        files_to_generate: &[String],
        config: &'a CodeGenConfig,
    ) -> TypeResolver<'a> {
        TypeResolver {
            context: CodeGenContext::for_generate(files, files_to_generate, config),
        }
    }

    /// The absolute Rust path of the message `proto_fqn` (`.pkg.Name`).
    /// Fails if the type is missing from the request, or if no mapping
    /// covers it.
    pub(crate) fn rust_path(&self, proto_fqn: &str) -> Result<String, Error> {
        // With every path absolute, the package of the referring file does
        // not change the result.
        let Some(path) = self.context.rust_type_relative(proto_fqn, "", 0) else {
            return Err(Error::new(format!(
                "type {proto_fqn} not found in the request (missing proto import?)"
            )));
        };
        if is_absolute(&path) {
            Ok(path)
        } else {
            Err(Error::new(format!(
                "type {proto_fqn} is not covered by `buffa_module` or any `extern_path`; \
                 add buffa_module=<path where the buffa output is mounted> \
                 (e.g. buffa_module=crate::proto) to the plugin parameters"
            )))
        }
    }

    /// Tokens of [`TypeResolver::rust_path`].
    pub(crate) fn rust_type(&self, proto_fqn: &str) -> Result<TokenStream, Error> {
        path_tokens(&self.rust_path(proto_fqn)?)
    }
}
