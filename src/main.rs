use std::{fs::File, io::Write, ops::RangeInclusive, os::raw};
use chrono::{Datelike, NaiveDate, Weekday};
use tera::{Context, Tera};
use std::collections::HashMap;
use serde::{Deserialize};


//Remove debug
#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
struct SheduleRawLesson {
    date_lesson: String,
    day_number: String,
    lesson_number: String,
    r#type: String,
    subject: String,
    teacher_name: String,
    place: String,
    subgroup: String
}

#[derive(Debug)]
struct SheduleLesson {
    position: u8,
    name: String,
    teacher: String,
    place: String
}

#[derive(Debug)]
struct SheduleDay {
    date: NaiveDate,
    day_of_week: u8,
    lessons: Vec<SheduleLesson>
}



fn raw_to_days(raw: Vec<SheduleRawLesson>) -> Vec<SheduleDay> {
    let mut shedule_days: Vec<SheduleDay> = vec![];

    let mut map: HashMap<String, Vec<SheduleRawLesson>> = HashMap::new();

    for raw_lesson in raw {
        map.entry(raw_lesson.date_lesson.clone())
        .or_insert_with(Vec::new)
        .push(raw_lesson)
    }

    let mut date_keys: Vec<String> = map.keys().into_iter().map(|str| str.to_owned()).collect();
    date_keys.sort();

    for lesson_date in &date_keys {
        let mut this_date_lessons: Vec<SheduleLesson> = vec![];

        for lesson in map.get(lesson_date).unwrap() {
            this_date_lessons.push(raw_to_lesson(lesson));
        }

        let day_date_str = lesson_date.split("-").collect::<Vec<&str>>();

        let day_date =
        NaiveDate::from_ymd_opt(
        day_date_str[0].parse().unwrap(),
        day_date_str[1].parse().unwrap(),
        day_date_str[2].parse().unwrap()
        ).unwrap();

        let lessons_day_of_week = match day_date.weekday() {
            Weekday::Mon => 1,
            Weekday::Tue => 2,
            Weekday::Wed => 3,
            Weekday::Thu => 4,
            Weekday::Fri => 5,
            Weekday::Sat => 6,
            Weekday::Sun => 7
        };

        shedule_days.push(SheduleDay { date: day_date.clone(), day_of_week: lessons_day_of_week, lessons: this_date_lessons });
    }

    shedule_days
}

fn raw_to_lesson(raw: &SheduleRawLesson) -> SheduleLesson {
    let subgroup = match raw.subgroup.parse::<u8>().unwrap() {
        1 => String::from(", 1 п/г"),
        2 => String::from(", 2 п/г"),
        _ => String::from("")
    };
    SheduleLesson {
        position: raw.lesson_number.parse().unwrap(),
        name: format!("{} {}{}", raw.r#type, raw.subject, subgroup),
        teacher: raw.teacher_name.clone(),
        place: raw.place.clone()
    }
}

fn days_to_weeks(days: Vec<SheduleDay>) -> Vec<Vec<SheduleDay>> {
    
    let mut weeks: Vec<Vec<SheduleDay>> = vec![vec![]];
    let mut weeks_counter: usize = 0;
    let mut week_dates: RangeInclusive<NaiveDate> = days[0].date.week(Weekday::Mon).days();

    for day in days {
        if week_dates.contains(&day.date) {
            weeks[weeks_counter].push(day);
        } else {
            weeks.push(vec![]);
            weeks_counter += 1;
            week_dates = day.date.week(Weekday::Mon).days();
            weeks[weeks_counter].push(day);
        }
    }

    weeks
}


fn main() {


    let response = reqwest::blocking::get("https://portal.kuzstu.ru/api/student_schedule?group_id=6668").unwrap();
    let shedule_classes = response.json::<Vec<SheduleRawLesson>>().unwrap();

    let group_name: String = String::from("ЦСб-231");

    let days = raw_to_days(shedule_classes);

    let weeks: Vec<Vec<SheduleDay>> = days_to_weeks(days);

    let mut data_file = File::create("./responses/shedule.txt").expect("No file?");
    data_file.write(format!("{:#?}", weeks).as_bytes()).expect("No data?");
    
    let tera = Tera::new("templates/*.html").unwrap();

    // Prepare the context with some data
    let mut context = Context::new();

    context.insert("group_name", &group_name);

    // Render the template with the given context
    let rendered = tera.render("index.html", &context).unwrap();

    print!("{}", rendered);
    
    
}