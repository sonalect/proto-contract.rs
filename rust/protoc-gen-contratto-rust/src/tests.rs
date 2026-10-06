//! Whole-request tests: descriptors built by hand, as protoc would send them.

use buffa::{Message, MessageField};
use buffa_codegen::generated::compiler::{CodeGeneratorRequest, CodeGeneratorResponse};
use buffa_codegen::generated::descriptor::source_code_info::Location;
use buffa_codegen::generated::descriptor::{
    DescriptorProto, FileDescriptorProto, MethodDescriptorProto, ServiceDescriptorProto,
    SourceCodeInfo,
};

use crate::{generate, run};

fn message(name: &str, nested: &[&str]) -> DescriptorProto {
    DescriptorProto {
        name: Some(name.into()),
        nested_type: nested.iter().map(|n| message(n, &[])).collect(),
        ..Default::default()
    }
}

fn file(name: &str, package: &str, messages: Vec<DescriptorProto>) -> FileDescriptorProto {
    FileDescriptorProto {
        name: Some(name.into()),
        package: Some(package.into()),
        message_type: messages,
        ..Default::default()
    }
}

fn method(name: &str, input: &str, output: &str) -> MethodDescriptorProto {
    MethodDescriptorProto {
        name: Some(name.into()),
        input_type: Some(input.into()),
        output_type: Some(output.into()),
        ..Default::default()
    }
}

/// `example/v1/greeter.proto` with its imports: a message of another
/// package, one of a package mapped by `extern_path`, and a well-known type.
fn greeter_files(methods: Vec<MethodDescriptorProto>) -> Vec<FileDescriptorProto> {
    let mut greeter = file(
        "example/v1/greeter.proto",
        "example.v1",
        vec![
            message("GreetRequest", &["Options"]),
            message("GreetReply", &[]),
        ],
    );
    greeter.dependency = vec![
        "example/common/v1/name.proto".into(),
        "example/shared/v1/label.proto".into(),
        "google/protobuf/empty.proto".into(),
    ];
    greeter.service = vec![ServiceDescriptorProto {
        name: Some("GreeterService".into()),
        method: methods,
        ..Default::default()
    }];
    greeter.source_code_info = MessageField::some(SourceCodeInfo {
        location: vec![
            Location {
                path: vec![6, 0],
                leading_comments: Some(" Greets people.\n".into()),
                ..Default::default()
            },
            Location {
                path: vec![6, 0, 2, 0],
                leading_comments: Some(" Greet a person by name.\n See <b>[Name]</b>.\n".into()),
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    vec![
        file(
            "google/protobuf/empty.proto",
            "google.protobuf",
            vec![message("Empty", &[])],
        ),
        file(
            "example/common/v1/name.proto",
            "example.common.v1",
            vec![message("Name", &[])],
        ),
        file(
            "example/shared/v1/label.proto",
            "example.shared.v1",
            vec![message("Label", &[])],
        ),
        greeter,
    ]
}

fn greeter_methods() -> Vec<MethodDescriptorProto> {
    vec![
        method(
            "Greet",
            ".example.v1.GreetRequest",
            ".example.v1.GreetReply",
        ),
        method(
            "GreetName",
            ".example.common.v1.Name",
            ".example.v1.GreetReply",
        ),
        method(
            "GetOptions",
            ".google.protobuf.Empty",
            ".example.v1.GreetRequest.Options",
        ),
        method(
            "EchoLabel",
            ".example.shared.v1.Label",
            ".example.shared.v1.Label",
        ),
        method("Type", ".google.protobuf.Empty", ".google.protobuf.Empty"),
    ]
}

fn request(files: Vec<FileDescriptorProto>, parameter: &str) -> CodeGeneratorRequest {
    CodeGeneratorRequest {
        file_to_generate: vec!["example/v1/greeter.proto".into()],
        parameter: Some(parameter.into()),
        proto_file: files,
        ..Default::default()
    }
}

/// `(name, content)` of every output file.
fn outputs(response: &CodeGeneratorResponse) -> Vec<(&str, &str)> {
    response
        .file
        .iter()
        .map(|f| (f.name.as_deref().unwrap(), f.content.as_deref().unwrap()))
        .collect()
}

/// The code without whitespace and without the trailing commas
/// prettyplease adds when it wraps a list, so checks do not depend on how
/// lines are wrapped.
fn squeeze(code: &str) -> String {
    code.split_whitespace()
        .collect::<String>()
        .replace(",)", ")")
        .replace(",>", ">")
}

const PARAMETER: &str = "buffa_module=crate::proto,extern_path=.example.shared=::shared_types";

#[test]
fn split_layout_emits_both_traits_and_a_stitcher() {
    let response = generate(&request(greeter_files(greeter_methods()), PARAMETER)).unwrap();
    assert_eq!(response.error, None);
    assert_eq!(response.supported_features, Some(3));
    assert_eq!(response.minimum_edition, Some(1000));
    assert_eq!(response.maximum_edition, Some(1001));

    let files = outputs(&response);
    let names: Vec<&str> = files.iter().map(|(name, _)| *name).collect();
    assert_eq!(
        names,
        ["example.v1.greeter.__contratto.rs", "example.v1.mod.rs"]
    );

    let (_, code) = files[0];
    assert!(code.starts_with(
        "// @generated by protoc-gen-contratto-rust. DO NOT EDIT.\n\
         // source: example/v1/greeter.proto\n"
    ));
    let code = squeeze(code);
    assert!(code.contains("pubtraitGreeterServiceSync:::core::marker::Send+::core::marker::Sync{"));
    assert!(
        code.contains("pubtraitGreeterServiceAsync:::core::marker::Send+::core::marker::Sync{")
    );
    assert!(code.contains(
        "fngreet(&self,request:crate::proto::example::v1::GreetRequest)\
         ->::core::result::Result<crate::proto::example::v1::GreetReply,::contratto::Status>;"
    ));
    assert!(code.contains(
        "fngreet(&self,request:crate::proto::example::v1::GreetRequest)\
         ->impl::core::future::Future<Output=::core::result::Result<crate::proto::example::v1::GreetReply,::contratto::Status>>+::core::marker::Send;"
    ));
    assert!(code.contains(
        "pubstructDynGreeterServiceAsync{inner:::std::sync::Arc<dyn__dyn_greeter_service_async::Erased>,}"
    ));
    assert!(code.contains("implGreeterServiceAsyncforDynGreeterServiceAsync{"));
    assert!(code.contains("mod__dyn_greeter_service_async{pubtraitErased:"));

    let (_, stitcher) = files[1];
    assert_eq!(
        stitcher,
        "// @generated by protoc-gen-contratto-rust. DO NOT EDIT.\n\
         include!(\"example.v1.greeter.__contratto.rs\");\n"
    );
}

/// T6: every kind of message path resolves to the path buffa gives it.
#[test]
fn message_paths_follow_buffa() {
    let response = generate(&request(greeter_files(greeter_methods()), PARAMETER)).unwrap();
    let code = squeeze(outputs(&response)[0].1);
    for expected in [
        // another package under `buffa_module`
        "fngreet_name(&self,request:crate::proto::example::common::v1::Name)",
        // a nested message
        "->::core::result::Result<crate::proto::example::v1::greet_request::Options,",
        // a package mapped by `extern_path`, longest prefix over `buffa_module`
        "fnecho_label(&self,request:::shared_types::v1::Label)",
        // a well-known type
        "fnr#type(&self,request:::buffa_types::google::protobuf::Empty)",
    ] {
        assert!(code.contains(expected), "missing {expected} in {code}");
    }
}

#[test]
fn comments_become_sanitized_rustdoc() {
    let response = generate(&request(greeter_files(greeter_methods()), PARAMETER)).unwrap();
    let code = outputs(&response)[0].1;
    assert!(
        code.contains("/// Greets people.\n///\n/// Blocking form of `example.v1.GreeterService`")
    );
    assert!(
        code.contains("/// Greets people.\n///\n/// Async form of `example.v1.GreeterService`")
    );
    assert!(
        code.contains("    /// Greet a person by name.\n    /// See \\<b\\>\\[Name\\]\\</b\\>.\n")
    );
    assert!(code.contains("    /// Call `example.v1.GreeterService.GetOptions`.\n"));
}

/// The buffa output may sit at the crate root.
#[test]
fn buffa_module_may_be_the_crate_root() {
    let response = generate(&request(
        greeter_files(greeter_methods()),
        "buffa_module=crate",
    ))
    .unwrap();
    let code = squeeze(outputs(&response)[0].1);
    assert!(
        code.contains("fngreet(&self,request:crate::example::v1::GreetRequest)"),
        "{code}"
    );
    assert!(code.contains("fnecho_label(&self,request:crate::example::shared::v1::Label)"));
}

/// A proto path cannot break out of the `// source:` comment.
#[test]
fn source_comment_escapes_line_breaks() {
    let mut files = greeter_files(greeter_methods());
    files[3].name = Some("evil\npub fn injected() {}.proto".into());
    let mut request = request(files, PARAMETER);
    request.file_to_generate = vec!["evil\npub fn injected() {}.proto".into()];
    let response = generate(&request).unwrap();
    let code = outputs(&response)[0].1;
    assert!(
        code.contains("// source: evil\\npub fn injected() {}.proto\n"),
        "{code}"
    );
    assert!(!code.contains("\npub fn injected"));
}

#[test]
fn runtime_path_is_a_parameter() {
    let parameter = format!("{PARAMETER},runtime=crate::rt");
    let mut methods = greeter_methods();
    methods.push(MethodDescriptorProto {
        client_streaming: Some(true),
        server_streaming: Some(true),
        ..method("Chat", ".example.v1.GreetRequest", ".example.v1.GreetReply")
    });
    let response = generate(&request(greeter_files(methods), &parameter)).unwrap();
    let code = squeeze(outputs(&response)[0].1);
    for item in ["Status", "BoxFuture", "BoxIter", "BoxStream", "Stream<"] {
        assert!(code.contains(&format!("crate::rt::{item}")), "{item}");
    }
    assert!(!code.contains("::contratto"));
}

#[test]
fn file_per_package_layout() {
    let mut files = greeter_files(greeter_methods());
    let mut counter = file(
        "example/v1/counter.proto",
        "example.v1",
        vec![message("AddRequest", &[])],
    );
    counter.service = vec![ServiceDescriptorProto {
        name: Some("CounterService".into()),
        method: vec![method(
            "Add",
            ".example.v1.AddRequest",
            ".example.v1.AddRequest",
        )],
        ..Default::default()
    }];
    files.push(counter);
    let mut request = request(files, &format!("{PARAMETER},file_per_package"));
    request
        .file_to_generate
        .push("example/v1/counter.proto".into());

    let response = generate(&request).unwrap();
    let files = outputs(&response);
    assert_eq!(files.len(), 1);
    let (name, code) = files[0];
    assert_eq!(name, "example.v1.rs");
    assert!(code.starts_with("// @generated by protoc-gen-contratto-rust. DO NOT EDIT.\n"));
    assert!(code.contains("// source: example/v1/greeter.proto\n"));
    assert!(code.contains("// source: example/v1/counter.proto\n"));
    assert!(code.contains("pub trait GreeterServiceSync"));
    assert!(code.contains("pub trait CounterServiceAsync"));
}

#[test]
fn files_without_services_produce_nothing() {
    let mut request = request(greeter_files(greeter_methods()), PARAMETER);
    request.file_to_generate = vec!["example/common/v1/name.proto".into()];
    let response = generate(&request).unwrap();
    assert!(response.file.is_empty());
}

/// T5: every kind of streaming method gets its signature in each form.
#[test]
fn streaming_methods_in_every_form() {
    let streaming = |name: &str, client: bool, server: bool| MethodDescriptorProto {
        client_streaming: Some(client),
        server_streaming: Some(server),
        ..method(name, ".example.v1.GreetRequest", ".example.v1.GreetReply")
    };
    let methods = vec![
        streaming("Watch", false, true),
        streaming("Collect", true, false),
        streaming("Chat", true, true),
    ];
    let response = generate(&request(greeter_files(methods), PARAMETER)).unwrap();
    let code = squeeze(outputs(&response)[0].1);

    let req = "crate::proto::example::v1::GreetRequest";
    let reply = "::core::result::Result<crate::proto::example::v1::GreetReply,::contratto::Status>";
    let item = format!("::core::result::Result<{req},::contratto::Status>");
    let send = "::core::marker::Send";
    let inbound = format!("whereR:::contratto::Stream<Item={item}>+{send}+'static");
    for expected in [
        // sync
        format!(
            "fnwatch(&self,request:{req})\
             ->::core::result::Result<::contratto::BoxIter<'static,{reply}>,::contratto::Status>;"
        ),
        format!("fncollect(&self,requests:::contratto::BoxIter<'static,{item}>)->{reply};"),
        format!(
            "fnchat(&self,requests:::contratto::BoxIter<'static,{item}>)\
             ->::core::result::Result<::contratto::BoxIter<'static,{reply}>,::contratto::Status>;"
        ),
        // async
        format!(
            "fnwatch(&self,request:{req})->impl::core::future::Future<Output=::core::result::Result<\
             impl::contratto::Stream<Item={reply}>+{send}+use<Self>,::contratto::Status>>+{send};"
        ),
        format!(
            "fncollect<R>(&self,requests:R)->impl::core::future::Future<Output={reply}>+{send}{inbound};"
        ),
        format!(
            "fnchat<R>(&self,requests:R)->impl::core::future::Future<Output=::core::result::Result<\
             impl::contratto::Stream<Item={reply}>+{send}+use<Self,R>,::contratto::Status>>+{send}{inbound};"
        ),
        // the erased face of the dyn handle
        format!(
            "fnchat(&self,requests:::contratto::BoxStream<'static,{item}>)->::contratto::BoxFuture<'_,\
             ::core::result::Result<::contratto::BoxStream<'static,{reply}>,::contratto::Status>>;"
        ),
    ] {
        assert!(code.contains(&expected), "missing {expected}\nin {code}");
    }
}

/// The bridges: each method of each kind forwards to the other form.
#[test]
fn bridges_for_every_kind() {
    let streaming = |name: &str, client: bool, server: bool| MethodDescriptorProto {
        client_streaming: Some(client),
        server_streaming: Some(server),
        ..method(name, ".example.v1.GreetRequest", ".example.v1.GreetReply")
    };
    let mut methods = greeter_methods();
    methods.push(streaming("Watch", false, true));
    methods.push(streaming("Collect", true, false));
    methods.push(streaming("Chat", true, true));
    let response = generate(&request(greeter_files(methods), PARAMETER)).unwrap();
    let code = squeeze(outputs(&response)[0].1);

    for expected in [
        "impl<T:GreeterServiceSync>GreeterServiceAsyncfor::contratto::Inline<T>{",
        "impl<T:GreeterServiceSync+'static>GreeterServiceAsyncfor::contratto::Offload<T>{",
        "impl<T:GreeterServiceAsync+'static>GreeterServiceSyncfor::contratto::Blocking<T>{",
        // Inline
        "asyncfngreet(&self,request:crate::proto::example::v1::GreetRequest)\
         ->::core::result::Result<crate::proto::example::v1::GreetReply,::contratto::Status>\
         {<TasGreeterServiceSync>::greet(Self::get_ref(self),request)}",
        "{<TasGreeterServiceSync>::watch(Self::get_ref(self),request).map(Self::reply_stream)}",
        "{letrequests=Self::buffer(requests).await;<TasGreeterServiceSync>::collect(Self::get_ref(self),requests)}",
        // Offload
        "Self::call(self,move|service|<TasGreeterServiceSync>::greet(service,request))",
        "Self::server_streaming(self,move|service|<TasGreeterServiceSync>::watch(service,request))",
        "Self::client_streaming(self,requests,<TasGreeterServiceSync>::collect)",
        "Self::bidirectional(self,requests,<TasGreeterServiceSync>::chat)",
        // Blocking
        "Self::block_on(self,<TasGreeterServiceAsync>::greet(Self::get_ref(self),request))",
        "Self::block_on_stream(self,<TasGreeterServiceAsync>::watch(Self::get_ref(self),request))",
        "letrequests=Self::feed(self,requests);Self::block_on(self,<TasGreeterServiceAsync>::collect(Self::get_ref(self),requests))",
        "letrequests=Self::feed(self,requests);Self::block_on_stream(self,<TasGreeterServiceAsync>::chat(Self::get_ref(self),requests))",
    ] {
        assert!(code.contains(expected), "missing {expected}\nin {code}");
    }
    assert!(!code.contains("#[cfg("));
}

#[test]
fn tokio_gate_covers_offload_and_blocking_only() {
    let parameter = format!("{PARAMETER},gate_tokio_feature=bridges");
    let response = generate(&request(greeter_files(greeter_methods()), &parameter)).unwrap();
    let code = squeeze(outputs(&response)[0].1);
    assert!(code.contains(
        "#[cfg(feature=\"bridges\")]impl<T:GreeterServiceSync+'static>GreeterServiceAsyncfor::contratto::Offload<T>"
    ));
    assert!(code.contains(
        "#[cfg(feature=\"bridges\")]impl<T:GreeterServiceAsync+'static>GreeterServiceSyncfor::contratto::Blocking<T>"
    ));
    assert_eq!(code.matches("#[cfg(").count(), 2);
}

#[test]
fn uncovered_type_names_the_fix() {
    let error = generate(&request(greeter_files(greeter_methods()), "")).unwrap_err();
    assert!(
        error
            .message()
            .contains("type .example.v1.GreetRequest is not covered"),
        "{error}"
    );
    assert!(
        error.message().contains("buffa_module=crate::proto"),
        "{error}"
    );
}

#[test]
fn missing_type_is_an_error() {
    let methods = vec![method(
        "Greet",
        ".example.v1.Missing",
        ".example.v1.GreetReply",
    )];
    let error = generate(&request(greeter_files(methods), PARAMETER)).unwrap_err();
    assert!(
        error
            .message()
            .contains("type .example.v1.Missing not found"),
        "{error}"
    );
}

#[test]
fn colliding_method_names_are_refused() {
    let methods = vec![
        method(
            "GetName",
            ".example.v1.GreetRequest",
            ".example.v1.GreetReply",
        ),
        method(
            "Get_Name",
            ".example.v1.GreetRequest",
            ".example.v1.GreetReply",
        ),
    ];
    let error = generate(&request(greeter_files(methods), PARAMETER)).unwrap_err();
    assert!(
        error
            .message()
            .contains("methods GetName and Get_Name both become"),
        "{error}"
    );
}

#[test]
fn clashing_trait_name_fails_the_run() {
    let mut files = greeter_files(greeter_methods());
    files[3]
        .message_type
        .push(message("GreeterServiceAsync", &[]));
    let response = run(&request(files, PARAMETER).encode_to_vec());
    let error = response.error.unwrap();
    assert!(
        error.contains("service example.v1.GreeterService"),
        "{error}"
    );
    assert!(
        error.contains("message example.v1.GreeterServiceAsync"),
        "{error}"
    );
}

#[test]
fn bad_input_and_parameters_reach_protoc_as_errors() {
    let response = run(&[0xff, 0xff, 0xff]);
    assert!(response.error.is_some());
    assert!(response.file.is_empty());

    let response = run(&request(greeter_files(greeter_methods()), "views=true").encode_to_vec());
    assert!(
        response
            .error
            .unwrap()
            .contains("unknown plugin parameter \"views=true\"")
    );
}
