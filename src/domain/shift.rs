use super::{MemberId, ValidationError};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub struct Shift {
    pub id: ShiftId,
    #[serde(skip_serializing)]
    pub member_id: MemberId,
    pub day: Day,
    #[serde(rename = "startTime")]
    pub start_time: Minute,
    #[serde(rename = "endTime")]
    pub end_time: Minute,
}

impl Shift {
    pub fn new(
        member_id: MemberId,
        day: Day,
        start_time: Minute,
        end_time: Minute,
    ) -> Result<Self, ValidationError> {
        validate_shift(&start_time, &end_time)?;

        Ok(Self {
            id: ShiftId::default(),
            member_id,
            day,
            start_time,
            end_time,
        })
    }

    pub fn length(&self) -> i16 {
        self.end_time.value_of() - self.start_time.value_of()
    }

    pub fn length_hours(&self) -> (i16, i16) {
        let minutes = self.end_time.value_of() - self.start_time.value_of();
        (minutes / 60, minutes % 60)
    }
}

fn validate_shift(
    start_time: &Minute,
    end_time: &Minute,
) -> Result<(), ValidationError> {
    if end_time.is_after(&start_time) {
        return Ok(());
    }
    Err(ValidationError::new(String::from(
        "Start time must be before end time",
    )))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShiftId(Uuid);

impl ShiftId {
    pub fn parse(id: &str) -> Result<Self, ValidationError> {
        let parsed = uuid::Uuid::try_parse(id).map_err(|e| {
            ValidationError::new(format!("Invalid member ID: {e}"))
        })?;
        Ok(Self(parsed))
    }

    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for ShiftId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl AsRef<Uuid> for ShiftId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

#[repr(i16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Day {
    Sunday = 0,
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
}

impl TryFrom<i16> for Day {
    type Error = ValidationError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Day::Sunday),
            1 => Ok(Day::Monday),
            2 => Ok(Day::Tuesday),
            3 => Ok(Day::Wednesday),
            4 => Ok(Day::Thursday),
            5 => Ok(Day::Friday),
            6 => Ok(Day::Saturday),
            _ => Err(ValidationError::new(String::from(
                "Invalid day of the week",
            ))),
        }
    }
}

impl From<Day> for i16 {
    fn from(day: Day) -> Self {
        day as i16
    }
}

impl FromStr for Day {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Sunday" | "sunday" => Ok(Day::Sunday),
            "Monday" | "monday" => Ok(Day::Monday),
            "Tuesday" | "tuesday" => Ok(Day::Tuesday),
            "Wednesday" | "wednesday" => Ok(Day::Wednesday),
            "Thursday" | "thursday" => Ok(Day::Thursday),
            "Friday" | "friday" => Ok(Day::Friday),
            "Saturday" | "saturday" => Ok(Day::Saturday),
            _ => Err(ValidationError::new(String::from("Invalid day"))),
        }
    }
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Day::Sunday => "Sunday",
                Day::Monday => "Monday",
                Day::Tuesday => "Tuesday",
                Day::Wednesday => "Wednesday",
                Day::Thursday => "Thursday",
                Day::Friday => "Friday",
                Day::Saturday => "Saturday",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Minute(i16);

const MINUTE_MIN: i16 = 0;
const MINUTE_MAX: i16 = 1440;

impl Minute {
    pub fn parse(minute: i16) -> Result<Self, ValidationError> {
        validate_minute(minute)?;
        Ok(Self(minute))
    }

    pub fn value_of(&self) -> i16 {
        self.0
    }

    pub fn is_after(&self, other: &Minute) -> bool {
        if self.value_of() > other.value_of() {
            return true;
        }
        false
    }

    pub fn is_before(&self, other: &Minute) -> bool {
        if self.value_of() < other.value_of() {
            return true;
        }
        false
    }

    pub fn to_hours(&self) -> (i16, i16) {
        (self.value_of() / 60, self.value_of() % 60)
    }

    pub fn difference(minute_one: &Minute, minute_two: &Minute) -> i16 {
        let num_one = minute_one.value_of();
        let num_two = minute_two.value_of();

        if num_one > num_two {
            return num_one - num_two;
        } else {
            return num_two - num_one;
        }
    }
}

fn validate_minute(num: i16) -> Result<(), ValidationError> {
    match num {
        num if num < MINUTE_MIN => Err(ValidationError::new(String::from(
            "Minute cannot be before midnight",
        ))),
        num if num > MINUTE_MAX => Err(ValidationError::new(String::from(
            "Minute cannot be after midnight",
        ))),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minute_parse() {
        assert!(Minute::parse(0).is_ok());
        assert!(Minute::parse(1440).is_ok());
        assert!(Minute::parse(1441).is_err());
        assert!(Minute::parse(i16::MAX).is_err());
        assert!(Minute::parse(-1).is_err());
        assert!(Minute::parse(i16::MAX).is_err());
    }

    #[test]
    fn test_minute_value_of() {
        assert!(Minute::parse(1).is_ok_and(|m| m.value_of() == 1));
        assert!(Minute::parse(1339).is_ok_and(|m| m.value_of() == 1339));
    }

    #[test]
    fn test_minute_is_before_and_after() {
        let first = Minute::parse(0).expect("Failed to parse minute");
        let second = Minute::parse(1).expect("Failed to parse minute");
        let third = Minute::parse(MINUTE_MAX).expect("Failed to parse minute");
        assert!(second.is_after(&first));
        assert!(third.is_after(&second));
        assert!(third.is_after(&first));
        assert!(first.is_before(&second));
        assert!(first.is_before(&third));
        assert!(second.is_before(&third));
    }

    #[test]
    fn test_minute_difference() {
        let zero = Minute::parse(0).expect("Failed to parse minute");
        let one = Minute::parse(1).expect("Failed to parse minute");
        let max = Minute::parse(MINUTE_MAX).expect("Failed to parse minute");

        assert_eq!(Minute::difference(&zero, &one), 1);
        assert_eq!(Minute::difference(&max, &zero), MINUTE_MAX);
    }

    #[test]
    fn test_valid_ids() {
        let valid_id = "5e90ca28-e1ad-4795-a190-089959c16e0b";
        let parsed = ShiftId::parse(valid_id).expect(valid_id);
        assert_eq!(
            parsed.as_ref().to_string(),
            valid_id,
            "ID does not match expected value"
        );
    }

    #[test]
    fn test_invalid_ids() {
        let invalid_id = "5b5b32e3a66cc-45bc-82d1-d41582139f1e";
        let result = ShiftId::parse(invalid_id);
        let error = result.expect_err(invalid_id);
        assert_eq!(error.as_ref(), "Invalid member ID: failed to parse a UUID");
    }

    #[test]
    fn test_shift_new() {
        let member_id = MemberId::default();
        let day = Day::Monday;
        let start_time =
            Minute::parse(540).expect("Failed to parse start_time");
        let end_time = Minute::parse(1020).expect("Failed to parse end_time");

        assert!(Shift::new(
            member_id.clone(),
            day,
            start_time.clone(),
            end_time.clone()
        )
        .is_ok());

        assert!(Shift::new(member_id, day, end_time, start_time).is_err());
    }

    #[test]
    fn test_shift_length() {
        let member_id = MemberId::default();
        let day = Day::Friday;
        let start_time =
            Minute::parse(540).expect("Failed to parse start time");
        let end_time = Minute::parse(1050).expect("Failed to parse end time");

        let shift = Shift::new(member_id, day, start_time, end_time)
            .expect("Failed to create shift");

        assert_eq!(shift.length(), 510);
        assert_eq!(shift.length_hours(), (8, 30));
    }
}
