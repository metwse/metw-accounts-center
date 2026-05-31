#[cfg(any(test, doc))]
pub(crate) mod mock;

#[cfg(any(test, doc))]
pub use mock::MockMailClientImpl;

#[cfg(any(test, doc))]
pub use mock::MockCaptchaClientImpl;
