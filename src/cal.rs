use std::ffi::CStr;

pub const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "Jun", "July", "August", "September", "October", "November", "December"];
pub const WEEKDAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

pub fn filler(year: u16, month: u8, day: &str) -> impl Fn(u8, u8) -> i8 {
    debug_assert!(year  != 0);
    debug_assert!(month != 0);

    let weekdays = WEEKDAYS.len() as u8;

    let days = days_in_month(year, month);
    let padding = (weekdays - first_day(day) - day_of_week(year, month, 1)) % weekdays;

    let days_prev = match month == 1 {
        false => days_in_month(year, month - 1),
        true  => days_in_month(year - 1, MONTHS.len() as u8),
    };

    move |column: u8, row: u8| {
        debug_assert!(column < weekdays);

        if (row <= (padding / weekdays)) && (column < padding) {
            return -((1 + days_prev + column - padding) as i8)
        }

        if ((row * weekdays) + column) >= days + padding {
            return -((1 + row * weekdays + column - padding - days) as i8)
        }

        (1 + (row * weekdays) + column - padding) as i8
    }
}

#[inline]
pub fn is_after_reform(year: u16) -> bool {
    debug_assert!(year  != 0);

    const REFORM_YEAR: u16 = 1752;
    const _REFORM_MONTH: u8 = 11;

    year > REFORM_YEAR
}

pub fn is_leap_year(year: u16) -> bool {
    debug_assert!(year  != 0);

    debug_assert!(is_after_reform(year));

    (year % 4 == 0) && (year % 100 != 0) || (year % 400 == 0)
}

pub fn days_in_month(year: u16, month: u8) -> u8 {
    debug_assert!(year  != 0);
    debug_assert!(month != 0);

    const DAYS:      [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    const DAYS_LEAP: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    match is_leap_year(year) {
        false => DAYS      [month as usize - 1],
        true  => DAYS_LEAP [month as usize - 1],
    }
}

pub fn day_of_week(mut year: u16, month: u8, day: u8) -> u8 {
    debug_assert!(year  != 0);
    debug_assert!(month != 0);
    debug_assert!(day   != 0);

    debug_assert!(is_after_reform(year));

    const OFFSETS: [u8; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];

    if month < 3 {
        year -= 1;
    }

    ((year + year / 4 - year / 100 + year / 400 + (OFFSETS[month as usize - 1] + day) as u16) % 7) as u8
}

#[inline]
pub fn first_day(day: &str) -> u8 {
    WEEKDAYS.iter()
        .position(|e| e.to_lowercase().find(day) == Some(0))
        .unwrap_or(0) as u8
}

pub fn weekdays_with_first(day: &str) -> [String; WEEKDAYS.len()] {
    let mut a = [const { String::new() }; WEEKDAYS.len()];
    a.iter_mut().enumerate().for_each(|(i, s)| *s = weekday(i as u8));
    a.rotate_left(first_day(day) as usize);
    a
}

fn weekday(day: u8) -> String {
    debug_assert!(day <= WEEKDAYS.len() as u8);

    let s = unsafe {
        let ptr = libc::nl_langinfo(libc::ABDAY_1 + day as i32);
        CStr::from_ptr(ptr)
    };

    s.to_owned().into_string().unwrap()
}

pub fn monthname(month: u8) -> String {
    debug_assert!(month <= MONTHS.len() as u8);
    debug_assert!(month != 0);

    // glibc
    const __ALTMON_1: libc::c_int = 131183;

    let s = unsafe {
        let ptr = libc::nl_langinfo(__ALTMON_1 - 1 + month as i32);
        CStr::from_ptr(ptr)
    };

    s.to_owned().into_string().unwrap()
}

pub fn date() -> (u16, u8, u8) {
    const LIBC_YEAR_FROM: u16 = 1900;

    unsafe {
        let time = libc::time(std::ptr::null_mut());
        let ptr = libc::localtime(&time as _);

        match ptr.is_null() {
            false => {
                let tm = *ptr;
                (LIBC_YEAR_FROM + tm.tm_year as u16, 1 + tm.tm_mon as u8, tm.tm_mday as u8)
            }
            true => (0, 0, 0)
        }
    }
}

