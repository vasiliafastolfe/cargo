extern crate cargotest;
extern crate hamcrest;

use cargotest::ChannelChanger;
use cargotest::support::registry::{self, Package, alt_api_path};
use cargotest::support::{project, execs};
use hamcrest::assert_that;

#[test]
fn is_feature_gated() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            version = "0.0.1"
            registry = "alternative"
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    Package::new("bar", "0.0.1").alternative(true).publish();

    assert_that(p.cargo("build").masquerade_as_nightly_cargo(),
                execs().with_status(101)
                .with_stderr_contains("  feature `alternative-registries` is required"));
}

#[test]
fn depend_on_alt_registry() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            cargo-features = ["alternative-registries"]

            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            version = "0.0.1"
            registry = "alternative"
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    Package::new("bar", "0.0.1").alternative(true).publish();

    assert_that(p.cargo("build").masquerade_as_nightly_cargo(),
                execs().with_status(0).with_stderr(&format!("\
[UPDATING] registry `{reg}`
[DOWNLOADING] bar v0.0.1 (registry `file://[..]`)
[COMPILING] bar v0.0.1 (registry `file://[..]`)
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..] secs
",
        dir = p.url(),
        reg = registry::alt_registry())));

    assert_that(p.cargo("clean").masquerade_as_nightly_cargo(), execs().with_status(0));

    // Don't download a second time
    assert_that(p.cargo("build").masquerade_as_nightly_cargo(),
                execs().with_status(0).with_stderr(&format!("\
[COMPILING] bar v0.0.1 (registry `file://[..]`)
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..] secs
",
        dir = p.url())));
}

#[test]
fn depend_on_alt_registry_depends_on_same_registry_no_index() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            cargo-features = ["alternative-registries"]

            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            version = "0.0.1"
            registry = "alternative"
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    Package::new("baz", "0.0.1").alternative(true).publish();
    Package::new("bar", "0.0.1").dep("baz", "0.0.1").alternative(true).publish();

    assert_that(p.cargo("build").masquerade_as_nightly_cargo(),
                execs().with_status(0).with_stderr(&format!("\
[UPDATING] registry `{reg}`
[DOWNLOADING] [..] v0.0.1 (registry `file://[..]`)
[DOWNLOADING] [..] v0.0.1 (registry `file://[..]`)
[COMPILING] baz v0.0.1 (registry `file://[..]`)
[COMPILING] bar v0.0.1 (registry `file://[..]`)
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..] secs
",
        dir = p.url(),
        reg = registry::alt_registry())));
}

#[test]
fn depend_on_alt_registry_depends_on_same_registry() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            cargo-features = ["alternative-registries"]

            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            version = "0.0.1"
            registry = "alternative"
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    Package::new("baz", "0.0.1").alternative(true).publish();
    Package::new("bar", "0.0.1").registry_dep("baz", "0.0.1", registry::alt_registry().as_str()).alternative(true).publish();

    assert_that(p.cargo("build").masquerade_as_nightly_cargo(),
                execs().with_status(0).with_stderr(&format!("\
[UPDATING] registry `{reg}`
[DOWNLOADING] [..] v0.0.1 (registry `file://[..]`)
[DOWNLOADING] [..] v0.0.1 (registry `file://[..]`)
[COMPILING] baz v0.0.1 (registry `file://[..]`)
[COMPILING] bar v0.0.1 (registry `file://[..]`)
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..] secs
",
        dir = p.url(),
        reg = registry::alt_registry())));
}

#[test]
fn depend_on_alt_registry_depends_on_crates_io() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            cargo-features = ["alternative-registries"]

            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            version = "0.0.1"
            registry = "alternative"
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    Package::new("baz", "0.0.1").publish();
    Package::new("bar", "0.0.1").registry_dep("baz", "0.0.1", registry::registry().as_str()).alternative(true).publish();

    assert_that(p.cargo("build").masquerade_as_nightly_cargo(),
                execs().with_status(0).with_stderr(&format!("\
[UPDATING] registry `{alt_reg}`
[UPDATING] registry `{reg}`
[DOWNLOADING] [..] v0.0.1 (registry `file://[..]`)
[DOWNLOADING] [..] v0.0.1 (registry `file://[..]`)
[COMPILING] baz v0.0.1 (registry `file://[..]`)
[COMPILING] bar v0.0.1 (registry `file://[..]`)
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..] secs
",
        dir = p.url(),
        alt_reg = registry::alt_registry(),
        reg = registry::registry())));
}

#[test]
fn registry_and_path_dep_works() {
    registry::init();

    let p = project("foo")
        .file("Cargo.toml", r#"
            cargo-features = ["alternative-registries"]

            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            path = "bar"
            registry = "alternative"
        "#)
        .file("src/main.rs", "fn main() {}")
        .file("bar/Cargo.toml", r#"
            [project]
            name = "bar"
            version = "0.0.1"
            authors = []
        "#)
        .file("bar/src/lib.rs", "")
        .build();

    assert_that(p.cargo("build").masquerade_as_nightly_cargo(),
                execs().with_status(0)
                .with_stderr(&format!("\
[COMPILING] bar v0.0.1 ({dir}/bar)
[COMPILING] foo v0.0.1 ({dir})
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..] secs
",
        dir = p.url())));
}

#[test]
fn registry_incompatible_with_git() {
    registry::init();

    let p = project("foo")
        .file("Cargo.toml", r#"
            cargo-features = ["alternative-registries"]

            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            git = ""
            registry = "alternative"
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    assert_that(p.cargo("build").masquerade_as_nightly_cargo(),
                execs().with_status(101)
                .with_stderr_contains("  dependency (bar) specification is ambiguous. Only one of `git` or `registry` is allowed."));
}

#[test]
fn cannot_publish_to_crates_io_with_registry_dependency() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            cargo-features = ["alternative-registries"]
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
            [dependencies.bar]
            version = "0.0.1"
            registry = "alternative"
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    Package::new("bar", "0.0.1").alternative(true).publish();

    assert_that(p.cargo("publish").masquerade_as_nightly_cargo()
                 .arg("--index").arg(registry::registry().to_string()),
                execs().with_status(101));
}

#[test]
fn publish_with_registry_dependency() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            cargo-features = ["alternative-registries"]

            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies.bar]
            version = "0.0.1"
            registry = "alternative"
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    Package::new("bar", "0.0.1").alternative(true).publish();

    // Login so that we have the token available
    assert_that(p.cargo("login").masquerade_as_nightly_cargo()
                 .arg("--registry").arg("alternative").arg("TOKEN").arg("-Zunstable-options"),
                execs().with_status(0));

    assert_that(p.cargo("publish").masquerade_as_nightly_cargo()
                 .arg("--registry").arg("alternative").arg("-Zunstable-options"),
                execs().with_status(0));
}

#[test]
fn alt_registry_and_crates_io_deps() {

    let p = project("foo")
        .file("Cargo.toml", r#"
            cargo-features = ["alternative-registries"]

            [project]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dependencies]
            crates_io_dep = "0.0.1"

            [dependencies.alt_reg_dep]
            version = "0.1.0"
            registry = "alternative"
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    Package::new("crates_io_dep", "0.0.1").publish();
    Package::new("alt_reg_dep", "0.1.0").alternative(true).publish();

    assert_that(p.cargo("build").masquerade_as_nightly_cargo(),
                execs().with_status(0)
                       .with_stderr_contains(format!("\
[UPDATING] registry `{}`", registry::alt_registry()))
                       .with_stderr_contains(&format!("\
[UPDATING] registry `{}`", registry::registry()))
                       .with_stderr_contains("\
[DOWNLOADING] crates_io_dep v0.0.1 (registry `file://[..]`)")
                       .with_stderr_contains("\
[DOWNLOADING] alt_reg_dep v0.1.0 (registry `file://[..]`)")
                       .with_stderr_contains("\
[COMPILING] alt_reg_dep v0.1.0 (registry `file://[..]`)")
                       .with_stderr_contains("\
[COMPILING] crates_io_dep v0.0.1")
                       .with_stderr_contains(&format!("\
[COMPILING] foo v0.0.1 ({})", p.url()))
                       .with_stderr_contains("\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..] secs"))

}

#[test]
fn block_publish_due_to_no_token() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    // Setup the registry by publishing a package
    Package::new("bar", "0.0.1").alternative(true).publish();

    // Now perform the actual publish
    assert_that(p.cargo("publish").masquerade_as_nightly_cargo()
                 .arg("--registry").arg("alternative").arg("-Zunstable-options"),
                execs().with_status(101)
                .with_stderr_contains("error: no upload token found, please run `cargo login`"));
}

#[test]
fn publish_to_alt_registry() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            cargo-features = ["alternative-registries"]

            [project]
            name = "foo"
            version = "0.0.1"
            authors = []
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    // Setup the registry by publishing a package
    Package::new("bar", "0.0.1").alternative(true).publish();

    // Login so that we have the token available
    assert_that(p.cargo("login").masquerade_as_nightly_cargo()
                 .arg("--registry").arg("alternative").arg("TOKEN").arg("-Zunstable-options"),
                execs().with_status(0));

    // Now perform the actual publish
    assert_that(p.cargo("publish").masquerade_as_nightly_cargo()
                 .arg("--registry").arg("alternative").arg("-Zunstable-options"),
                execs().with_status(0));

    // Ensure that the crate is uploaded
    assert!(alt_api_path().join("api/v1/crates/new").exists());
}

#[test]
fn publish_with_crates_io_dep() {
    let p = project("foo")
        .file("Cargo.toml", r#"
            cargo-features = ["alternative-registries"]

            [project]
            name = "foo"
            version = "0.0.1"
            authors = ["me"]
            license = "MIT"
            description = "foo"

            [dependencies.bar]
            version = "0.0.1"
        "#)
        .file("src/main.rs", "fn main() {}")
        .build();

    Package::new("bar", "0.0.1").publish();

    // Login so that we have the token available
    assert_that(p.cargo("login").masquerade_as_nightly_cargo()
                 .arg("--registry").arg("alternative").arg("TOKEN").arg("-Zunstable-options"),
                execs().with_status(0));

    assert_that(p.cargo("publish").masquerade_as_nightly_cargo()
                 .arg("--registry").arg("alternative").arg("-Zunstable-options"),
                execs().with_status(0));
}
