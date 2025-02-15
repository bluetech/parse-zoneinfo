use std::str::FromStr;
// we still support rust that doesn't have the inherent methods
#[allow(deprecated, unused_imports)]
use std::ascii::AsciiExt;

use regex::{Regex, Captures};

pub struct LineParser {
    rule_line: Regex,
    day_field: Regex,
    hm_field: Regex,
    hms_field: Regex,
    zone_line: Regex,
    continuation_line: Regex,
    link_line: Regex,
    empty_line: Regex,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Error {
    FailedYearParse(String),
    FailedMonthParse(String),
    FailedWeekdayParse(String),
    InvalidLineType(String),
    TypeColumnContainedNonHyphen(String),
    CouldNotParseSaving(String),
    InvalidDaySpec(String),
    InvalidTimeSpecAndType(String),
    NonWallClockInTimeSpec(String),
    NotParsedAsRuleLine,
    NotParsedAsZoneLine,
    NotParsedAsLinkLine,
}

impl LineParser {
    pub fn new() -> Self {
        LineParser {
            rule_line: Regex::new(r##"(?x) ^
                Rule \s+
                ( ?P<name>    \S+)  \s+
                ( ?P<from>    \S+)  \s+
                ( ?P<to>      \S+)  \s+
                ( ?P<type>    \S+)  \s+
                ( ?P<in>      \S+)  \s+
                ( ?P<on>      \S+)  \s+
                ( ?P<at>      \S+)  \s+
                ( ?P<save>    \S+)  \s+
                ( ?P<letters> \S+)  \s*
                (\#.*)?
            $ "##).unwrap(),

            day_field: Regex::new(r##"(?x) ^
                ( ?P<weekday> \w+ )
                ( ?P<sign>    [<>] = )
                ( ?P<day>     \d+ )
            $ "##).unwrap(),

            hm_field: Regex::new(r##"(?x) ^
                ( ?P<sign> -? )
                ( ?P<hour> \d{1,2} ) : ( ?P<minute> \d{2} )
                ( ?P<flag> [wsugz] )?
            $ "##).unwrap(),

            hms_field: Regex::new(r##"(?x) ^
                ( ?P<sign> -? )
                ( ?P<hour> \d{1,2} ) : ( ?P<minute> \d{2} ) : ( ?P<second> \d{2} )
                ( ?P<flag> [wsugz] )?
            $ "##).unwrap(),

            zone_line: Regex::new(r##"(?x) ^
                Zone \s+
                ( ?P<name> [ A-Z a-z 0-9 / _ + - ]+ )  \s+
                ( ?P<gmtoff>     \S+ )  \s+
                ( ?P<rulessave>  \S+ )  \s+
                ( ?P<format>     \S+ )  \s*
                ( ?P<year>       \S+ )? \s*
                ( ?P<month>      \S+ )? \s*
                ( ?P<day>        \S+ )? \s*
                ( ?P<time>       \S+ )? \s*
                (\#.*)?
            $ "##).unwrap(),

            continuation_line: Regex::new(r##"(?x) ^
                \s+
                ( ?P<gmtoff>     \S+ )  \s+
                ( ?P<rulessave>  \S+ )  \s+
                ( ?P<format>     \S+ )  \s*
                ( ?P<year>       \S+ )? \s*
                ( ?P<month>      \S+ )? \s*
                ( ?P<day>        \S+ )? \s*
                ( ?P<time>       \S+ )? \s*
                (\#.*)?
            $ "##).unwrap(),

            link_line: Regex::new(r##"(?x) ^
                Link  \s+
                ( ?P<target>  \S+ )  \s+
                ( ?P<name>    \S+ )  \s*
                (\#.*)?
            $ "##).unwrap(),

            empty_line: Regex::new(r##"(?x) ^
                \s*
                (\#.*)?
            $"##).unwrap(),
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Year {
    Minimum,
    Maximum,
    Number(i64),
}

impl FromStr for Year {
    type Err = Error;

    fn from_str(input: &str) -> Result<Year, Self::Err> {
        Ok(match &*input.to_ascii_lowercase() {
            "min" | "minimum" => Year::Minimum,
            "max" | "maximum" => Year::Maximum,
            year => match year.parse() {
                Ok(year) => Year::Number(year),
                Err(_)   => return Err(Error::FailedYearParse(input.to_string())),
            }
        })
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl Month {
    fn length(self, is_leap: bool) -> i8 {
        match self {
            Month::January             => 31,
            Month::February if is_leap => 29,
            Month::February            => 28,
            Month::March               => 31,
            Month::April               => 30,
            Month::May                 => 31,
            Month::June                => 30,
            Month::July                => 31,
            Month::August              => 31,
            Month::September           => 30,
            Month::October             => 31,
            Month::November            => 30,
            Month::December            => 31,
        }
    }
}

impl FromStr for Month {
    type Err = Error;

    fn from_str(input: &str) -> Result<Month, Self::Err> {
        Ok(match &*input.to_ascii_lowercase() {
            "jan" | "january"    => Month::January,
            "feb" | "february"   => Month::February,
            "mar" | "march"      => Month::March,
            "apr" | "april"      => Month::April,
            "may"                => Month::May,
            "jun" | "june"       => Month::June,
            "jul" | "july"       => Month::July,
            "aug" | "august"     => Month::August,
            "sep" | "september"  => Month::September,
            "oct" | "october"    => Month::October,
            "nov" | "november"   => Month::November,
            "dec" | "december"   => Month::December,
            other                => return Err(Error::FailedMonthParse(other.to_string())),
        })
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Weekday {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl FromStr for Weekday {
    type Err = Error;

    fn from_str(input: &str) -> Result<Weekday, Self::Err> {
        Ok(match &*input.to_ascii_lowercase() {
            "mon" | "monday"     => Weekday::Monday,
            "tue" | "tuesday"    => Weekday::Tuesday,
            "wed" | "wednesday"  => Weekday::Wednesday,
            "thu" | "thursday"   => Weekday::Thursday,
            "fri" | "friday"     => Weekday::Friday,
            "sat" | "saturday"   => Weekday::Saturday,
            "sun" | "sunday"     => Weekday::Sunday,
            other                => return Err(Error::FailedWeekdayParse(other.to_string())),
        })
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum DaySpec {
    Ordinal(i8),
    Last(Weekday),
    LastOnOrBefore(Weekday, i8),
    FirstOnOrAfter(Weekday, i8)
}

impl Weekday {
    fn calculate(year: i64, month: Month, day: i8) -> Weekday {
        let m = month as i64;
        let y = if m < 3 { year - 1} else { year };
        let d = day as i64;
        const T: [i64; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        match (y + y/4 - y/100 + y/400 + T[m as usize-1] + d) % 7 {
            0 => Weekday::Sunday,
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            6 => Weekday::Saturday,
            _ => panic!("why is negative modulus designed so?")
        }
    }
}

#[cfg(test)]
#[test]
fn weekdays() {
    assert_eq!(Weekday::calculate(1970, Month::January, 1), Weekday::Thursday);
    assert_eq!(Weekday::calculate(2017, Month::February, 11), Weekday::Saturday);
    assert_eq!(Weekday::calculate(1890, Month::March, 2), Weekday::Sunday);
    assert_eq!(Weekday::calculate(2100, Month::April, 20), Weekday::Tuesday);
    assert_eq!(Weekday::calculate(2009, Month::May, 31), Weekday::Sunday);
    assert_eq!(Weekday::calculate(2001, Month::June, 9), Weekday::Saturday);
    assert_eq!(Weekday::calculate(1995, Month::July, 21), Weekday::Friday);
    assert_eq!(Weekday::calculate(1982, Month::August, 8), Weekday::Sunday);
    assert_eq!(Weekday::calculate(1962, Month::September, 6), Weekday::Thursday);
    assert_eq!(Weekday::calculate(1899, Month::October, 14), Weekday::Saturday);
    assert_eq!(Weekday::calculate(2016, Month::November, 18), Weekday::Friday);
    assert_eq!(Weekday::calculate(2010, Month::December, 19), Weekday::Sunday);
    assert_eq!(Weekday::calculate(2016, Month::February, 29), Weekday::Monday);
}

fn is_leap(year: i64) -> bool {
    // Leap year rules: years which are factors of 4, except those divisible
    // by 100, unless they are divisible by 400.
    //
    // We test most common cases first: 4th year, 100th year, then 400th year.
    //
    // We factor out 4 from 100 since it was already tested, leaving us checking
    // if it's divisible by 25. Afterwards, we do the same, factoring 25 from
    // 400, leaving us with 16.
    //
    // Factors of 4 and 16 can quickly be found with bitwise AND.
    year & 3 == 0 && (year % 25 != 0 || year & 15 == 0)
}

#[cfg(test)]
#[test]
fn leap_years() {
    assert!(!is_leap(1900));
    assert!(is_leap(1904));
    assert!(is_leap(1964));
    assert!(is_leap(1996));
    assert!(!is_leap(1997));
    assert!(!is_leap(1997));
    assert!(!is_leap(1999));
    assert!(is_leap(2000));
    assert!(is_leap(2016));
    assert!(!is_leap(2100));
}

impl DaySpec {
    pub fn to_concrete_day(&self, year: i64, month: Month) -> i8 {
        let length = month.length(is_leap(year));

        match *self {
            DaySpec::Ordinal(day) => day,
            DaySpec::Last(weekday) => (1..length+1).rev()
                .find(|&day| Weekday::calculate(year, month, day) == weekday).unwrap(),
            DaySpec::LastOnOrBefore(weekday, day) => (1..day+1).rev()
                .find(|&day| Weekday::calculate(year, month, day) == weekday).unwrap(),
            DaySpec::FirstOnOrAfter(weekday, day) => (day..length+1)
                .find(|&day| Weekday::calculate(year, month, day) == weekday).unwrap(),
        }
    }
}

#[cfg(test)]
#[test]
fn last_monday() {
    let dayspec = DaySpec::Last(Weekday::Monday);
    assert_eq!(dayspec.to_concrete_day(2016, Month::January), 25);
    assert_eq!(dayspec.to_concrete_day(2016, Month::February), 29);
    assert_eq!(dayspec.to_concrete_day(2016, Month::March), 28);
    assert_eq!(dayspec.to_concrete_day(2016, Month::April), 25);
    assert_eq!(dayspec.to_concrete_day(2016, Month::May), 30);
    assert_eq!(dayspec.to_concrete_day(2016, Month::June), 27);
    assert_eq!(dayspec.to_concrete_day(2016, Month::July), 25);
    assert_eq!(dayspec.to_concrete_day(2016, Month::August), 29);
    assert_eq!(dayspec.to_concrete_day(2016, Month::September), 26);
    assert_eq!(dayspec.to_concrete_day(2016, Month::October), 31);
    assert_eq!(dayspec.to_concrete_day(2016, Month::November), 28);
    assert_eq!(dayspec.to_concrete_day(2016, Month::December), 26);
}

#[cfg(test)]
#[test]
fn first_monday_on_or_after() {
    let dayspec = DaySpec::FirstOnOrAfter(Weekday::Monday, 20);
    assert_eq!(dayspec.to_concrete_day(2016, Month::January), 25);
    assert_eq!(dayspec.to_concrete_day(2016, Month::February), 22);
    assert_eq!(dayspec.to_concrete_day(2016, Month::March), 21);
    assert_eq!(dayspec.to_concrete_day(2016, Month::April), 25);
    assert_eq!(dayspec.to_concrete_day(2016, Month::May), 23);
    assert_eq!(dayspec.to_concrete_day(2016, Month::June), 20);
    assert_eq!(dayspec.to_concrete_day(2016, Month::July), 25);
    assert_eq!(dayspec.to_concrete_day(2016, Month::August), 22);
    assert_eq!(dayspec.to_concrete_day(2016, Month::September), 26);
    assert_eq!(dayspec.to_concrete_day(2016, Month::October), 24);
    assert_eq!(dayspec.to_concrete_day(2016, Month::November), 21);
    assert_eq!(dayspec.to_concrete_day(2016, Month::December), 26);
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimeSpec {
    Hours(i8),
    HoursMinutes(i8, i8),
    HoursMinutesSeconds(i8, i8, i8),
    Zero,
}

impl TimeSpec {
    pub fn as_seconds(self) -> i64 {
        match self {
            TimeSpec::Hours(h) => h as i64 * 60 * 60,
            TimeSpec::HoursMinutes(h, m) => h as i64 * 60 * 60 + m as i64 * 60,
            TimeSpec::HoursMinutesSeconds(h, m, s) => h as i64 * 60 * 60 + m as i64 * 60 + s as i64,
            TimeSpec::Zero => 0,
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TimeType {
    Wall,
    Standard,
    UTC,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct TimeSpecAndType(pub TimeSpec, pub TimeType);

impl TimeSpec {
    pub fn with_type(self, timetype: TimeType) -> TimeSpecAndType {
        TimeSpecAndType(self, timetype)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ChangeTime {
    UntilYear(Year),
    UntilMonth(Year, Month),
    UntilDay(Year, Month, DaySpec),
    UntilTime(Year, Month, DaySpec, TimeSpecAndType),
}

impl ChangeTime {
    pub fn to_timestamp(&self) -> i64 {

        fn seconds_in_year(year: i64) -> i64 {
            if is_leap(year) {
                366 * 24 * 60 * 60
            } else {
                365 * 24 * 60 * 60
            }
        }

        fn seconds_until_start_of_year(year: i64) -> i64 {
            if year >= 1970 {
                (1970..year).map(seconds_in_year).sum()
            } else {
                -(year..1970).map(seconds_in_year).sum::<i64>()
            }
        }

        fn time_to_timestamp(year: i64, month: i8, day: i8, hour: i8, minute: i8, second: i8) -> i64 {
            const MONTHS_NON_LEAP: [i64; 12] = [
                0,
                31,
                31 + 28,
                31 + 28 + 31,
                31 + 28 + 31 + 30,
                31 + 28 + 31 + 30 + 31,
                31 + 28 + 31 + 30 + 31 + 30,
                31 + 28 + 31 + 30 + 31 + 30 + 31,
                31 + 28 + 31 + 30 + 31 + 30 + 31 + 31,
                31 + 28 + 31 + 30 + 31 + 30 + 31 + 31 + 30,
                31 + 28 + 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31,
                31 + 28 + 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31 + 30
            ];
            const MONTHS_LEAP: [i64; 12] = [
                0,
                31,
                31 + 29,
                31 + 29 + 31,
                31 + 29 + 31 + 30,
                31 + 29 + 31 + 30 + 31,
                31 + 29 + 31 + 30 + 31 + 30,
                31 + 29 + 31 + 30 + 31 + 30 + 31,
                31 + 29 + 31 + 30 + 31 + 30 + 31 + 31,
                31 + 29 + 31 + 30 + 31 + 30 + 31 + 31 + 30,
                31 + 29 + 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31,
                31 + 29 + 31 + 30 + 31 + 30 + 31 + 31 + 30 + 31 + 30
            ];
            seconds_until_start_of_year(year)
                + 60 * 60 * 24 * if is_leap(year) { MONTHS_LEAP[month as usize - 1] } else { MONTHS_NON_LEAP[month as usize - 1] }
                + 60 * 60 * 24 * (day as i64 - 1)
                + 60 * 60 * hour as i64
                + 60 * minute as i64
                + second as i64
        }

        match *self {
            ChangeTime::UntilYear(Year::Number(y))             => time_to_timestamp(y, 1,                             1, 0, 0,   0),
            ChangeTime::UntilMonth(Year::Number(y), m)         => time_to_timestamp(y, m as i8,                       1, 0, 0,   0),
            ChangeTime::UntilDay(Year::Number(y), m, d)        => time_to_timestamp(y, m as i8, d.to_concrete_day(y, m), 0, 0,   0),
            ChangeTime::UntilTime(Year::Number(y), m, d, time) => match time.0 {
                TimeSpec::Zero                                 => time_to_timestamp(y, m as i8, d.to_concrete_day(y, m), 0, 0,   0),
                TimeSpec::Hours(h)                             => time_to_timestamp(y, m as i8, d.to_concrete_day(y, m), h, 0,   0),
                TimeSpec::HoursMinutes(h, min)                 => time_to_timestamp(y, m as i8, d.to_concrete_day(y, m), h, min, 0),
                TimeSpec::HoursMinutesSeconds(h, min, s)       => time_to_timestamp(y, m as i8, d.to_concrete_day(y, m), h, min, s),
            },
            _ => unreachable!(),
        }
    }

    pub fn year(&self) -> i64 {
        match *self {
            ChangeTime::UntilYear(Year::Number(y))      => y,
            ChangeTime::UntilMonth(Year::Number(y), ..) => y,
            ChangeTime::UntilDay(Year::Number(y), ..)   => y,
            ChangeTime::UntilTime(Year::Number(y), ..)  => y,
            _ => unreachable!()
        }
    }
}

#[cfg(test)]
#[test]
fn to_timestamp() {
    let time = ChangeTime::UntilYear(Year::Number(1970));
    assert_eq!(time.to_timestamp(), 0);
    let time = ChangeTime::UntilYear(Year::Number(2016));
    assert_eq!(time.to_timestamp(), 1451606400);
    let time = ChangeTime::UntilYear(Year::Number(1900));
    assert_eq!(time.to_timestamp(), -2208988800);
    let time = ChangeTime::UntilTime(Year::Number(2000), Month::February, DaySpec::Last(Weekday::Sunday),
        TimeSpecAndType(TimeSpec::Hours(9), TimeType::Wall));
    assert_eq!(time.to_timestamp(), 951642000);
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct ZoneInfo<'a> {
    pub utc_offset: TimeSpec,
    pub saving: Saving<'a>,
    pub format: &'a str,
    pub time: Option<ChangeTime>,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Saving<'a> {
    NoSaving,
    OneOff(TimeSpec),
    Multiple(&'a str),
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Rule<'a> {
    pub name: &'a str,
    pub from_year: Year,
    pub to_year: Option<Year>,
    pub month: Month,
    pub day: DaySpec,
    pub time: TimeSpecAndType,
    pub time_to_add: TimeSpec,
    pub letters: Option<&'a str>,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Zone<'a> {
    pub name: &'a str,
    pub info: ZoneInfo<'a>,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Link<'a> {
    pub existing: &'a str,
    pub new: &'a str,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Line<'a> {
    Space,
    Zone(Zone<'a>),
    Continuation(ZoneInfo<'a>),
    Rule(Rule<'a>),
    Link(Link<'a>),
}

fn parse_time_type(c: &str) -> Option<TimeType> {
    Some(match c {
        "w"             => TimeType::Wall,
        "s"             => TimeType::Standard,
        "u" | "g" | "z" => TimeType::UTC,
         _              => return None,
    })
}

impl LineParser {
    fn parse_timespec_and_type(&self, input: &str) -> Result<TimeSpecAndType, Error> {
        if input == "-" {
            Ok(TimeSpecAndType(TimeSpec::Zero, TimeType::Wall))
        }
        else if input.chars().all(|c| c == '-' || c.is_digit(10)) {
            Ok(TimeSpecAndType(TimeSpec::Hours(input.parse().unwrap()), TimeType::Wall))
        }
        else if let Some(caps) = self.hm_field.captures(input) {
            let sign   : i8 = if caps.name("sign").unwrap().as_str() == "-" { -1 } else { 1 };
            let hour   : i8 = caps.name("hour").unwrap().as_str().parse().unwrap();
            let minute : i8 = caps.name("minute").unwrap().as_str().parse().unwrap();
            let flag        = caps.name("flag").and_then(|c| parse_time_type(&c.as_str()[0..1]))
                                          .unwrap_or(TimeType::Wall);

            Ok(TimeSpecAndType(TimeSpec::HoursMinutes(hour * sign, minute * sign), flag))
        }
        else if let Some(caps) = self.hms_field.captures(input) {
            let sign   : i8 = if caps.name("sign").unwrap().as_str() == "-" { -1 } else { 1 };
            let hour   : i8 = caps.name("hour").unwrap().as_str().parse().unwrap();
            let minute : i8 = caps.name("minute").unwrap().as_str().parse().unwrap();
            let second : i8 = caps.name("second").unwrap().as_str().parse().unwrap();
            let flag        = caps.name("flag").and_then(|c| parse_time_type(&c.as_str()[0..1]))
                                          .unwrap_or(TimeType::Wall);

            Ok(TimeSpecAndType(TimeSpec::HoursMinutesSeconds(hour * sign, minute * sign, second * sign), flag))
        } else {
            Err(Error::InvalidTimeSpecAndType(input.to_string()))
        }
    }

    fn parse_timespec(&self, input: &str) -> Result<TimeSpec, Error> {
        match self.parse_timespec_and_type(input) {
            Ok(TimeSpecAndType(spec, TimeType::Wall)) => Ok(spec),
            Ok(TimeSpecAndType(_, _)) => Err(Error::NonWallClockInTimeSpec(input.to_string())),
            Err(e) => Err(e),
        }
    }

    fn parse_dayspec(&self, input: &str) -> Result<DaySpec, Error> {
        if input.chars().all(|c| c.is_digit(10)) {
            Ok(DaySpec::Ordinal(input.parse().unwrap()))
        } else if input.starts_with("last") {
            let weekday = input[4..].parse()?;
            Ok(DaySpec::Last(weekday))
        } else if let Some(caps) = self.day_field.captures(input) {
            let weekday = caps.name("weekday").unwrap().as_str().parse().unwrap();
            let day     = caps.name("day").unwrap().as_str().parse().unwrap();

            match caps.name("sign").unwrap().as_str() {
                "<=" => Ok(DaySpec::LastOnOrBefore(weekday, day)),
                ">=" => Ok(DaySpec::FirstOnOrAfter(weekday, day)),
                 _   => unreachable!("The regex only matches one of those two!"),
            }
        } else {
            Err(Error::InvalidDaySpec(input.to_string()))
        }
    }

    fn parse_rule<'a>(&self, input: &'a str) -> Result<Rule<'a>, Error> {
        if let Some(caps) = self.rule_line.captures(input) {
            let name      = caps.name("name").unwrap().as_str();
            let from_year = caps.name("from").unwrap().as_str().parse()?;

            // The end year can be ‘only’ to indicate that this rule only
            // takes place on that year.
            let to_year = match caps.name("to").unwrap().as_str() {
                "only"  => None,
                to      => Some(to.parse()?),
            };

            // According to the spec, the only value inside the ‘type’ column
            // should be “-”, so throw an error if it isn’t. (It only exists
            // for compatibility with old versions that used to contain year
            // types.) Sometimes “‐”, a Unicode hyphen, is used as well.
            let t = caps.name("type").unwrap().as_str();
            if t != "-" && t != "\u{2010}"  {
                return Err(Error::TypeColumnContainedNonHyphen(t.to_string()));
            }

            let month        = caps.name("in").unwrap().as_str().parse()?;
            let day          = self.parse_dayspec(caps.name("on").unwrap().as_str())?;
            let time         = self.parse_timespec_and_type(caps.name("at").unwrap().as_str())?;
            let time_to_add  = self.parse_timespec(caps.name("save").unwrap().as_str())?;
            let letters      = match caps.name("letters").unwrap().as_str() {
                "-"  => None,
                l    => Some(l),
            };

            Ok(Rule {
                name:         name,
                from_year:    from_year,
                to_year:      to_year,
                month:        month,
                day:          day,
                time:         time,
                time_to_add:  time_to_add,
                letters:      letters,
            })
        } else {
            Err(Error::NotParsedAsRuleLine)
        }
    }

    fn saving_from_str<'a>(&self, input: &'a str) -> Result<Saving<'a>, Error> {
        if input == "-" {
            Ok(Saving::NoSaving)
        } else if input.chars().all(|c| c == '-' || c == '_' || c.is_alphabetic()) {
            Ok(Saving::Multiple(input))
        } else if self.hm_field.is_match(input) {
            let time = self.parse_timespec(input)?;
            Ok(Saving::OneOff(time))
        } else {
            Err(Error::CouldNotParseSaving(input.to_string()))
        }
    }

    fn zoneinfo_from_captures<'a>(&self, caps: Captures<'a>) -> Result<ZoneInfo<'a>, Error> {
        let utc_offset = self.parse_timespec(caps.name("gmtoff").unwrap().as_str())?;
        let saving = self.saving_from_str(caps.name("rulessave").unwrap().as_str())?;
        let format = caps.name("format").unwrap().as_str();

        let time = match (caps.name("year"), caps.name("month"), caps.name("day"), caps.name("time")) {
            (Some(y), Some(m), Some(d), Some(t)) => Some(ChangeTime::UntilTime  (y.as_str().parse()?, m.as_str().parse()?, self.parse_dayspec(d.as_str())?, self.parse_timespec_and_type(t.as_str())?)),
            (Some(y), Some(m), Some(d), _      ) => Some(ChangeTime::UntilDay   (y.as_str().parse()?, m.as_str().parse()?, self.parse_dayspec(d.as_str())?)),
            (Some(y), Some(m), _      , _      ) => Some(ChangeTime::UntilMonth (y.as_str().parse()?, m.as_str().parse()?)),
            (Some(y), _      , _      , _      ) => Some(ChangeTime::UntilYear  (y.as_str().parse()?)),
            (None   , None   , None   , None   ) => None,
            _                                    => unreachable!("Out-of-order capturing groups!"),
        };

        Ok(ZoneInfo {
            utc_offset:  utc_offset,
            saving:      saving,
            format:      format,
            time:        time,
        })
    }

    fn parse_zone<'a>(&self, input: &'a str) -> Result<Zone<'a>, Error> {
        if let Some(caps) = self.zone_line.captures(input) {
            let name = caps.name("name").unwrap().as_str();
            let info = self.zoneinfo_from_captures(caps)?;
            Ok(Zone {
                name: name,
                info: info,
            })
        } else {
            Err(Error::NotParsedAsZoneLine)
        }
    }

    fn parse_link<'a>(&self, input: &'a str) -> Result<Link<'a>, Error> {
        if let Some(caps) = self.link_line.captures(input) {
            let target  = caps.name("target").unwrap().as_str();
            let name    = caps.name("name").unwrap().as_str();
            Ok(Link { existing: target, new: name })
        }
        else {
            Err(Error::NotParsedAsLinkLine)
        }
    }

    pub fn parse_str<'a>(&self, input: &'a str) -> Result<Line<'a>, Error> {
        if self.empty_line.is_match(input) {
            return Ok(Line::Space)
        }

        match self.parse_zone(input) {
            Err(Error::NotParsedAsZoneLine) => {},
            result => return result.map(Line::Zone),
        }

        match self.continuation_line.captures(input) {
            None => {},
            Some(caps) => return self.zoneinfo_from_captures(caps).map(Line::Continuation),
        }

        match self.parse_rule(input) {
            Err(Error::NotParsedAsRuleLine) => {},
            result => return result.map(Line::Rule),
        }

        match self.parse_link(input) {
            Err(Error::NotParsedAsLinkLine) => {},
            result => return result.map(Line::Link),
        }
        
        Err(Error::InvalidLineType(input.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test {
        ($name:ident: $input:expr => $result:expr) => {
            #[test]
            fn $name() {
                let parser = LineParser::new();
                assert_eq!(parser.parse_str($input), $result);
            }
        };
    }

    test!(empty:    ""          => Ok(Line::Space));
    test!(spaces:   "        "  => Ok(Line::Space));

    test!(rule_1: "Rule  US    1967  1973  ‐     Apr  lastSun  2:00  1:00  D" => Ok(Line::Rule(Rule {
        name:         "US",
        from_year:    Year::Number(1967),
        to_year:      Some(Year::Number(1973)),
        month:        Month::April,
        day:          DaySpec::Last(Weekday::Sunday),
        time:         TimeSpec::HoursMinutes(2, 0).with_type(TimeType::Wall),
        time_to_add:  TimeSpec::HoursMinutes(1, 0),
        letters:      Some("D"),
    })));

    test!(rule_2: "Rule	Greece	1976	only	-	Oct	10	2:00s	0	-" => Ok(Line::Rule(Rule {
        name:         "Greece",
        from_year:    Year::Number(1976),
        to_year:      None,
        month:        Month::October,
        day:          DaySpec::Ordinal(10),
        time:         TimeSpec::HoursMinutes(2, 0).with_type(TimeType::Standard),
        time_to_add:  TimeSpec::Hours(0),
        letters:      None,
    })));

    test!(rule_3: "Rule	EU	1977	1980	-	Apr	Sun>=1	 1:00u	1:00	S" => Ok(Line::Rule(Rule {
        name:         "EU",
        from_year:    Year::Number(1977),
        to_year:      Some(Year::Number(1980)),
        month:        Month::April,
        day:          DaySpec::FirstOnOrAfter(Weekday::Sunday, 1),
        time:         TimeSpec::HoursMinutes(1, 0).with_type(TimeType::UTC),
        time_to_add:  TimeSpec::HoursMinutes(1, 0),
        letters:      Some("S"),
    })));

    test!(no_hyphen: "Rule	EU	1977	1980	HEY	Apr	Sun>=1	 1:00u	1:00	S"         => Err(Error::TypeColumnContainedNonHyphen("HEY".to_string())));
    test!(bad_month: "Rule	EU	1977	1980	-	Febtober	Sun>=1	 1:00u	1:00	S" => Err(Error::FailedMonthParse("febtober".to_string())));

    test!(zone: "Zone  Australia/Adelaide  9:30    Aus         AC%sT   1971 Oct 31  2:00:00" => Ok(Line::Zone(Zone {
        name: "Australia/Adelaide",
        info: ZoneInfo {
            utc_offset:  TimeSpec::HoursMinutes(9, 30),
            saving:      Saving::Multiple("Aus"),
            format:      "AC%sT",
            time:        Some(ChangeTime::UntilTime(Year::Number(1971), Month::October, DaySpec::Ordinal(31), TimeSpec::HoursMinutesSeconds(2, 0, 0).with_type(TimeType::Wall))),
        },
    })));

    test!(continuation_1: "                          9:30    Aus         AC%sT   1971 Oct 31  2:00:00" => Ok(Line::Continuation(ZoneInfo {
        utc_offset:  TimeSpec::HoursMinutes(9, 30),
        saving:      Saving::Multiple("Aus"),
        format:      "AC%sT",
        time:        Some(ChangeTime::UntilTime(Year::Number(1971), Month::October, DaySpec::Ordinal(31), TimeSpec::HoursMinutesSeconds(2, 0, 0).with_type(TimeType::Wall))),
    })));

    test!(continuation_2: "			1:00	C-Eur	CE%sT	1943 Oct 25" => Ok(Line::Continuation(ZoneInfo {
        utc_offset:  TimeSpec::HoursMinutes(1, 00),
        saving:      Saving::Multiple("C-Eur"),
        format:      "CE%sT",
        time:        Some(ChangeTime::UntilDay(Year::Number(1943), Month::October, DaySpec::Ordinal(25))),
    })));

    test!(zone_hyphen: "Zone Asia/Ust-Nera\t 9:32:54 -\tLMT\t1919" => Ok(Line::Zone(Zone {
        name: "Asia/Ust-Nera",
        info: ZoneInfo {
            utc_offset:  TimeSpec::HoursMinutesSeconds(9, 32, 54),
            saving:      Saving::NoSaving,
            format:      "LMT",
            time:        Some(ChangeTime::UntilYear(Year::Number(1919))),
        },
    })));

    #[test]
    fn negative_offsets() {
        static LINE: &'static str = "Zone    Europe/London   -0:01:15 -  LMT 1847 Dec  1  0:00s";
        let parser = LineParser::new();
        let zone = parser.parse_zone(LINE).unwrap();
        assert_eq!(zone.info.utc_offset, TimeSpec::HoursMinutesSeconds(0, -1, -15));
    }

    #[test]
    fn negative_offsets_2() {
        static LINE: &'static str = "Zone        Europe/Madrid   -0:14:44 -      LMT     1901 Jan  1  0:00s";
        let parser = LineParser::new();
        let zone = parser.parse_zone(LINE).unwrap();
        assert_eq!(zone.info.utc_offset, TimeSpec::HoursMinutesSeconds(0, -14, -44));
    }

    #[test]
    fn negative_offsets_3() {
        static LINE: &'static str = "Zone America/Danmarkshavn -1:14:40 -    LMT 1916 Jul 28";
        let parser = LineParser::new();
        let zone = parser.parse_zone(LINE).unwrap();
        assert_eq!(zone.info.utc_offset, TimeSpec::HoursMinutesSeconds(-1, -14, -40));
    }

    test!(link: "Link  Europe/Istanbul  Asia/Istanbul" => Ok(Line::Link(Link {
        existing:  "Europe/Istanbul",
        new:       "Asia/Istanbul",
    })));

    #[test]
    fn month() {
        assert_eq!(Month::from_str("Aug"), Ok(Month::August));
        assert_eq!(Month::from_str("December"), Ok(Month::December));
    }

    test!(golb: "GOLB" => Err(Error::InvalidLineType("GOLB".to_string())));

    test!(comment: "# this is a comment" => Ok(Line::Space));
    test!(another_comment: "     # so is this" => Ok(Line::Space));
    test!(multiple_hash: "     # so is this ## " => Ok(Line::Space));
    test!(non_comment: " this is not a # comment" => Err(Error::InvalidTimeSpecAndType("this".to_string())));

    test!(comment_after: "Link  Europe/Istanbul  Asia/Istanbul #with a comment after" => Ok(Line::Link(Link {
        existing:  "Europe/Istanbul",
        new:       "Asia/Istanbul",
    })));

    test!(two_comments_after: "Link  Europe/Istanbul  Asia/Istanbul   # comment ## comment" => Ok(Line::Link(Link {
        existing:  "Europe/Istanbul",
        new:       "Asia/Istanbul",
    })));
}
