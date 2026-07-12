//! Build scripts.

mod email_templates;

/// Build scripts.
fn main() {
    println!("cargo:rerun-if-changed=build");

    email_templates::minimize_email_templates();
}
