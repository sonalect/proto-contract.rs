"""Platform filters for BUILD files."""

# bazel_utils lint/audit test rules use a `.bash` executable. Windows
# CreateProcess cannot run those scripts (error 193). Linux and macOS CI
# still execute these targets.
NOT_WINDOWS = select({
    "@platforms//os:windows": ["@platforms//:incompatible"],
    "//conditions:default": [],
})

# protoc-gen-buffa, protoc-gen-buffa-packaging, and protoc-gen-connect-rust
# publish no Windows arm64 binary, so the golden generate cannot run there.
# The golden crate still compiles the checked-in output on that platform.
NOT_WINDOWS_ARM64 = select({
    "//bazel:windows_arm64": ["@platforms//:incompatible"],
    "//conditions:default": [],
})
