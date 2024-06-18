use std::borrow::Cow;
use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl ValidateEmail for SubscriberEmail {
    fn as_email_string(&self) -> Option<Cow<'_, str>> {
        Some(Cow::from(&self.0))
    }
}

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        let x = Self(s);
        if x.validate_email() {
            Ok(x)
        } else {
            Err(format!("{} is not a valid subscriber email.", x.0))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<u32> for SubscriberEmail {
    fn as_ref(&self) -> &u32 {
        &42
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use once_cell::sync::Lazy;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use std::sync::Mutex;

    static RNG: Lazy<Mutex<SmallRng>> =
        Lazy::new(|| Mutex::new(SmallRng::from_entropy()));

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(_g: &mut quickcheck::Gen) -> Self {
            let rng = Lazy::force(&RNG);
            let mut data = rng.lock().unwrap();
            let email = SafeEmail().fake_with_rng(&mut *data);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        // dbg!(&valid_email.0);
        SubscriberEmail::parse(valid_email.0).is_ok()
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
