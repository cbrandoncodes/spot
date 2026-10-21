use std::str::FromStr;

use garde::Validate;
use thiserror::Error;

const RESERVED_USERNAMES: &[&str] = &[
    "admin",
    "api",
    "help",
    "moderator",
    "spot",
    "support",
    "system",
];

#[derive(Clone, Debug, Default)]
pub struct ProfileChanges {
    pub name: Option<ProfileName>,
    pub username: Option<Username>,
    pub bio: Option<Option<Bio>>,
    pub country_code: Option<Option<CountryCode>>,
    pub city: Option<Option<City>>,
    pub district: Option<Option<District>>,
}

#[derive(Clone, Debug, Validate)]
pub struct Username {
    #[garde(ascii, length(bytes, min = 3, max = 30))]
    display: String,
    #[garde(skip)]
    normalized: String,
}

impl Username {
    pub fn normalized(&self) -> &str {
        &self.normalized
    }

    pub fn into_parts(self) -> (String, String) {
        (self.display, self.normalized)
    }
}

impl TryFrom<String> for Username {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let display = value.trim().to_owned();
        let normalized = display.to_ascii_lowercase();
        let username = Self {
            display,
            normalized,
        };

        if username.validate().is_err() {
            return Err(ValidationError::new(
                "username",
                "must be 3 to 30 ASCII characters",
            ));
        }

        let valid_character =
            |character: char| character.is_ascii_alphanumeric() || matches!(character, '.' | '_');
        if !username.display.chars().all(valid_character)
            || !username
                .display
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphanumeric())
            || !username
                .display
                .chars()
                .next_back()
                .is_some_and(|character| character.is_ascii_alphanumeric())
            || username.display.contains("..")
            || username.display.contains("__")
            || username.display.contains("._")
            || username.display.contains("_.")
        {
            return Err(ValidationError::new(
                "username",
                "must start and end with a letter or number and cannot repeat separators",
            ));
        }
        if RESERVED_USERNAMES.contains(&username.normalized.as_str()) {
            return Err(ValidationError::new("username", "is reserved"));
        }

        Ok(username)
    }
}

impl TryFrom<&str> for Username {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl FromStr for Username {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

macro_rules! text_value {
    ($name:ident, $field:literal, $maximum:literal) => {
        #[derive(Clone, Debug, Validate)]
        #[garde(transparent)]
        pub struct $name(#[garde(length(chars, min = 1, max = $maximum))] String);

        impl $name {
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = ValidationError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                let value = Self(value.trim().to_owned());
                value.validate().map_err(|_| {
                    ValidationError::new(
                        $field,
                        concat!("must contain between 1 and ", $maximum, " characters"),
                    )
                })?;
                Ok(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = ValidationError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::try_from(value.to_owned())
            }
        }

        impl FromStr for $name {
            type Err = ValidationError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::try_from(value)
            }
        }
    };
}

text_value!(ProfileName, "name", 80);
text_value!(Bio, "bio", 160);
text_value!(City, "city", 80);
text_value!(District, "district", 80);

#[derive(Clone, Debug, Validate)]
#[garde(transparent)]
pub struct CountryCode(#[garde(ascii, length(bytes, equal = 2))] String);

impl CountryCode {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl TryFrom<String> for CountryCode {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = Self(value.trim().to_ascii_uppercase());
        if value.validate().is_err()
            || !value
                .0
                .chars()
                .all(|character| character.is_ascii_alphabetic())
        {
            return Err(ValidationError::new(
                "countryCode",
                "must be a two-letter country code",
            ));
        }
        Ok(value)
    }
}

impl TryFrom<&str> for CountryCode {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl FromStr for CountryCode {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    Public,
    Private,
}

impl FromStr for Visibility {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            _ => Err(ValidationError::new(
                "visibility",
                "must be `public` or `private`",
            )),
        }
    }
}

impl Visibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FriendRequestPolicy {
    Anyone,
    FriendsOfFriends,
    SameGym,
    NoOne,
}

impl FromStr for FriendRequestPolicy {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "anyone" => Ok(Self::Anyone),
            "friends_of_friends" => Ok(Self::FriendsOfFriends),
            "same_gym" => Ok(Self::SameGym),
            "no_one" => Ok(Self::NoOne),
            _ => Err(ValidationError::new(
                "friendRequestPolicy",
                "must be `anyone`, `friends_of_friends`, `same_gym`, or `no_one`",
            )),
        }
    }
}

impl FriendRequestPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Anyone => "anyone",
            Self::FriendsOfFriends => "friends_of_friends",
            Self::SameGym => "same_gym",
            Self::NoOne => "no_one",
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{field} {message}")]
pub struct ValidationError {
    pub field: &'static str,
    pub message: String,
}

impl ValidationError {
    fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bio, CountryCode, FriendRequestPolicy, ProfileName, Username, Visibility};

    #[test]
    fn username_normalization_is_ascii_and_predictable() {
        let username: Username = "  Brandon.C  ".parse().expect("valid username");
        assert_eq!(username.clone().into_parts().0, "Brandon.C");
        assert_eq!(username.normalized(), "brandon.c");
    }

    #[test]
    fn rejects_reserved_and_ambiguous_usernames() {
        assert!("spot".parse::<Username>().is_err());
        assert!("first..last".parse::<Username>().is_err());
        assert!("_name".parse::<Username>().is_err());
        assert!("námé".parse::<Username>().is_err());
    }

    #[test]
    fn country_codes_are_normalized() {
        let country: CountryCode = " ng ".parse().expect("valid country");
        assert_eq!(country.into_inner(), "NG");
        assert!("NGA".parse::<CountryCode>().is_err());
    }

    #[test]
    fn text_value_objects_trim_and_enforce_character_limits() {
        let name = ProfileName::try_from("  Ada  ".to_owned()).expect("valid name");
        assert_eq!(name.into_inner(), "Ada");
        assert!(ProfileName::try_from(" ".to_owned()).is_err());
        assert!(Bio::try_from("🏋️".repeat(161)).is_err());
    }

    #[test]
    fn textual_enums_use_standard_parsing() {
        assert_eq!("public".parse::<Visibility>(), Ok(Visibility::Public));
        assert_eq!(
            "same_gym".parse::<FriendRequestPolicy>(),
            Ok(FriendRequestPolicy::SameGym)
        );
        assert!("followers".parse::<FriendRequestPolicy>().is_err());
    }
}
