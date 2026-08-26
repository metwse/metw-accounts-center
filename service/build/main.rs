//! Build scripts.

#[cfg(feature = "email-templates")]
mod email_templates;

/// Build scripts.
fn main() {
    println!("cargo:rerun-if-changed=build");

    #[cfg(feature = "email-templates")]
    email_templates::minimize_email_templates();
}
