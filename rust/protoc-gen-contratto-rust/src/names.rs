//! Names of the generated items, and the check that they are free.
//!
//! Contratto's output is often mounted into the module that already holds
//! a package's buffa messages and connect-rust's service items. Two items
//! with one name in one module do not compile (E0428), so the run fails
//! first and names both proto elements.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use buffa_codegen::generated::descriptor::{FileDescriptorProto, ServiceDescriptorProto};
use heck::{ToSnakeCase, ToUpperCamelCase};

use crate::Error;

/// The names Contratto gives the items of one service.
pub(crate) struct ServiceNames {
    /// The blocking trait, `<Service>Sync`.
    pub(crate) sync: String,
    /// The async trait, `<Service>Async`.
    pub(crate) asynchronous: String,
    /// The dynamic-dispatch handle, `Dyn<Service>Async`.
    pub(crate) dynamic: String,
    /// The private module of the handle's erased trait,
    /// `__dyn_<service>_async`.
    pub(crate) erased_module: String,
}

impl ServiceNames {
    pub(crate) fn of(service: &ServiceDescriptorProto) -> ServiceNames {
        let base = service
            .name
            .as_deref()
            .unwrap_or_default()
            .to_upper_camel_case();
        let asynchronous = format!("{base}Async");
        ServiceNames {
            sync: format!("{base}Sync"),
            dynamic: format!("Dyn{asynchronous}"),
            erased_module: format!("__dyn_{}", asynchronous.to_snake_case()),
            asynchronous,
        }
    }

    fn all(self) -> [String; 4] {
        [
            self.sync,
            self.asynchronous,
            self.dynamic,
            self.erased_module,
        ]
    }
}

/// The Rust method name of an RPC, as `protoc-gen-connect-rust` makes it.
pub(crate) fn method_name(rpc: &str) -> String {
    rpc.to_snake_case()
}

/// `package.Name`, or `Name` in the empty package.
pub(crate) fn qualified(package: &str, name: &str) -> String {
    if package.is_empty() {
        name.to_string()
    } else {
        format!("{package}.{name}")
    }
}

/// Fail if a name Contratto gives an item of a service in `package` is
/// also the name of another item of the package's Rust module.
///
/// Items checked: messages and enums (buffa), the server trait and the `Ext`, `RegisterMarker`, `Server`, and `Client`
/// items of every service (connect-rust), and the items of every other
/// service (Contratto). connect-rust's constants are upper snake case and
/// its `Owned…View` aliases end in `View`, so neither can match a name that
/// ends in `Sync` or `Async`.
pub(crate) fn check_clashes(files: &[FileDescriptorProto], package: &str) -> Result<(), Error> {
    let in_package = || {
        files
            .iter()
            .filter(move |file| file.package.as_deref().unwrap_or_default() == package)
    };

    let mut taken: HashMap<String, String> = HashMap::new();
    for file in in_package() {
        for message in &file.message_type {
            let name = message.name.as_deref().unwrap_or_default();
            taken.insert(
                name.to_string(),
                format!("message {}", qualified(package, name)),
            );
        }
        for enumeration in &file.enum_type {
            let name = enumeration.name.as_deref().unwrap_or_default();
            taken.insert(
                name.to_string(),
                format!("enum {}", qualified(package, name)),
            );
        }
        for service in &file.service {
            let name = service.name.as_deref().unwrap_or_default();
            let base = name.to_upper_camel_case();
            let origin = format!("service {} (connect-rust)", qualified(package, name));
            let server = if base == "Self" {
                "Self_".to_string()
            } else {
                base.clone()
            };
            for item in [
                server,
                format!("{base}Ext"),
                format!("{base}RegisterMarker"),
                format!("{base}Server"),
                format!("{base}Client"),
            ] {
                taken.insert(item, origin.clone());
            }
        }
    }

    for file in in_package() {
        for service in &file.service {
            let name = service.name.as_deref().unwrap_or_default();
            let origin = format!("service {}", qualified(package, name));
            for item in ServiceNames::of(service).all() {
                match taken.entry(item) {
                    Entry::Vacant(entry) => {
                        entry.insert(origin.clone());
                    }
                    Entry::Occupied(entry) => {
                        return Err(Error::new(format!(
                            "{origin}: generated item `{}` has the name of {} in the \
                             same Rust module; rename one of them in the proto",
                            entry.key(),
                            entry.get()
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use buffa_codegen::generated::descriptor::{
        DescriptorProto, EnumDescriptorProto, FileDescriptorProto, ServiceDescriptorProto,
    };

    use super::{ServiceNames, check_clashes, method_name};

    fn service(name: &str) -> ServiceDescriptorProto {
        ServiceDescriptorProto {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    fn file(package: &str, messages: &[&str], services: &[&str]) -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some(format!("{package}/f.proto")),
            package: Some(package.into()),
            message_type: messages
                .iter()
                .map(|name| DescriptorProto {
                    name: Some((*name).into()),
                    ..Default::default()
                })
                .collect(),
            service: services.iter().map(|name| service(name)).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn names_follow_connect_rust() {
        assert_eq!(
            ServiceNames::of(&service("GreeterService")).all(),
            [
                "GreeterServiceSync",
                "GreeterServiceAsync",
                "DynGreeterServiceAsync",
                "__dyn_greeter_service_async",
            ]
        );
        assert_eq!(
            ServiceNames::of(&service("greeter")).all(),
            [
                "GreeterSync",
                "GreeterAsync",
                "DynGreeterAsync",
                "__dyn_greeter_async"
            ]
        );
        assert_eq!(method_name("GetOptions"), "get_options");
        assert_eq!(method_name("HTTPGet"), "http_get");
    }

    #[test]
    fn distinct_names_pass() {
        let files = [file(
            "a.v1",
            &["FooRequest", "FooReply"],
            &["FooService", "BarService"],
        )];
        assert!(check_clashes(&files, "a.v1").is_ok());
    }

    #[test]
    fn clash_with_a_message_names_both() {
        let files = [file("a.v1", &[], &["Foo"]), file("a.v1", &["FooSync"], &[])];
        let error = check_clashes(&files, "a.v1").unwrap_err();
        assert!(error.message().contains("service a.v1.Foo"), "{error}");
        assert!(error.message().contains("`FooSync`"), "{error}");
        assert!(error.message().contains("message a.v1.FooSync"), "{error}");
    }

    #[test]
    fn clash_with_an_enum() {
        let mut files = [file("a.v1", &[], &["Foo"])];
        files[0].enum_type.push(EnumDescriptorProto {
            name: Some("FooAsync".into()),
            ..Default::default()
        });
        let error = check_clashes(&files, "a.v1").unwrap_err();
        assert!(error.message().contains("enum a.v1.FooAsync"), "{error}");
    }

    #[test]
    fn clash_with_connect_rust_server_trait() {
        let files = [file("a.v1", &[], &["Foo", "FooAsync"])];
        let error = check_clashes(&files, "a.v1").unwrap_err();
        assert!(error.message().contains("service a.v1.Foo:"), "{error}");
        assert!(
            error
                .message()
                .contains("service a.v1.FooAsync (connect-rust)"),
            "{error}"
        );
    }

    #[test]
    fn clash_between_two_services_of_contratto() {
        let files = [file("a.v1", &[], &["foo", "Foo"])];
        let error = check_clashes(&files, "a.v1").unwrap_err();
        assert!(error.message().contains("`FooSync`"), "{error}");
    }

    #[test]
    fn clash_with_the_dyn_handle() {
        let files = [file("a.v1", &["DynFooAsync"], &["Foo"])];
        let error = check_clashes(&files, "a.v1").unwrap_err();
        assert!(error.message().contains("`DynFooAsync`"), "{error}");
    }

    #[test]
    fn other_packages_do_not_count() {
        let files = [file("a.v1", &[], &["Foo"]), file("b.v1", &["FooSync"], &[])];
        assert!(check_clashes(&files, "a.v1").is_ok());
    }
}
