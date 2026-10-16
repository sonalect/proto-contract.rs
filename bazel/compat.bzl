"""Platform filters for BUILD files."""

# On Windows rules_rust links doc tests against a sysroot path that
# link.exe cannot open (LNK1181); `cargo test` runs them there.
NOT_WINDOWS = select({
    "@platforms//os:windows": ["@platforms//:incompatible"],
    "//conditions:default": [],
})
