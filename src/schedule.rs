use std::collections::HashMap;
use anyhow::format_err;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use crate::get_html;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupScheduleField {
    pub subject: String,
    pub teacher: String,
    pub classroom: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleField {
    Group(HashMap<u8, GroupScheduleField>),
    Class(GroupScheduleField),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRow {
    pub time: String,
    pub monday: ScheduleField,
    pub tuesday: ScheduleField,
    pub wednesday: ScheduleField,
    pub thursday: ScheduleField,
    pub friday: ScheduleField,
}

pub type Schedule = Vec<ScheduleRow>;

pub async fn get_schedule(schedule_url: &str) -> Schedule {
    let html = get_html(schedule_url).await.unwrap();
    let document = Html::parse_document(&html);
    
    get_schedule_from_html(document)
}

fn get_schedule_from_html(document: Html) -> Schedule {

    let table_selector = Selector::parse("table.tabela").unwrap();
    let table = document.select(&table_selector).next().unwrap();
    
    let tr_selector = Selector::parse("tr")
        .map_err(|e| format_err!(e.to_string())).unwrap();

    let mut schedule = vec![];
    
    let trs = table.select(&tr_selector);
    for tr in trs {
        if let Some(schedule_row) = get_schedule_row(tr) {
            schedule.push(schedule_row);
        }
    }

    schedule
}

fn get_schedule_row(tr: scraper::ElementRef) -> Option<ScheduleRow> {
    // If a row doesn't have td with class "g" it means it's not a schedule row
    let g_class_selector = Selector::parse("td.g").ok()?;

    let time = tr.select(&g_class_selector).next()?
        .text().collect::<Vec<_>>().join(" ")
        .replace(" ", "");

    let mut schedule_fields: Vec<ScheduleField> = vec![];

    let l_class_selector = Selector::parse("td.l").ok()?;
    for td in tr.select(&l_class_selector) {
        let text = td.text().collect::<Vec<_>>().join(" ");
        let text = text.trim().to_string();

        if text.is_empty() || text == "&nbsp;" {
            schedule_fields.push(ScheduleField::Empty);
            continue;
        }

        let span_selector = Selector::parse("td > span").ok()?;
        let spans = td.select(&span_selector);

        let s_class_a_selector = Selector::parse("a.s").ok()?;
        let s_class_a_elements = td.select(&s_class_a_selector);

        let n_class_a_selector = Selector::parse("a.n").ok()?;
        let n_class_a_elements = td.select(&n_class_a_selector);

        let p_class_selector = Selector::parse("span.p").ok()?;
        let p_spans_count = td.select(&p_class_selector).count();


        if p_spans_count == 0 {
            schedule_fields.push(ScheduleField::Empty);
            continue;
        }

        if p_spans_count == 1 {
            schedule_fields.push(ScheduleField::Class(
                get_group_schedule_field(Html::parse_fragment(&td.html()))
            ));
            continue;
        }

        let mut group_schedule = HashMap::new();
        for (group_num, span) in spans.clone().enumerate() {
            let mut combined = "".to_string();

            // if span has class != "p" then skip above logic
            if let Some(class) = span.value().attr("class") {
                if !class.contains("p") {
                    combined = span.html();
                }
            } else {
                combined = span.html();
            }

            if combined == "" {
                // combine span with a.s and a.n
                let span_str = span.html();
                let a_s = s_class_a_elements.clone()
                    .nth(group_num)?.html();
                let a_n = n_class_a_elements.clone()
                    .nth(group_num)?.html();

                combined = span_str + &a_n + &a_s;
            }

            let mut schedule_field = get_group_schedule_field(Html::parse_fragment(&combined));

            let split_subject = schedule_field.subject.split("-")
                .collect::<Vec<_>>();

            let mut group_num: u8 = 0;
            let split_subject_len = split_subject.len();

            if split_subject_len > 1 {
                group_num = split_subject.last()?.split("/").next()?
                    .parse().ok()?;
                
                schedule_field.subject = split_subject[0..split_subject_len-1]
                    .join("");
            }

            group_schedule.insert(
                group_num,
                schedule_field,
            );
        }

        schedule_fields.push(ScheduleField::Group(group_schedule));
    }


    Some(ScheduleRow {
        time,
        monday: schedule_fields[0].clone(),
        tuesday: schedule_fields[1].clone(),
        wednesday: schedule_fields[2].clone(),
        thursday: schedule_fields[3].clone(),
        friday: schedule_fields[4].clone(),
    })
}

fn get_group_schedule_field(single_group_html: Html) -> GroupScheduleField {
    // the subject is <span class="p">...</span>
    let p_class_selector = Selector::parse("span.p").unwrap();
    let subject = single_group_html.select(&p_class_selector).next().unwrap()
        .text().collect::<Vec<_>>().join(" ")
        .trim().to_string();

    // the teacher is <a class="n">...</a>
    let a_class_selector = Selector::parse("a.n").unwrap();
    let teacher = single_group_html.select(&a_class_selector).next().unwrap()
        .text().collect::<Vec<_>>().join(" ")
        .trim().to_string();

    // the teacher_id is href of <a class="n">...</a>
    let _teacher_id = single_group_html.select(&a_class_selector).next().unwrap()
        .value().attr("href").unwrap()
        .split('.').next().unwrap()
        .to_string();

    // the classroom is <a class="s">...</a>
    let s_class_selector = Selector::parse("a.s").unwrap();
    let classroom = single_group_html.select(&s_class_selector).next().unwrap()
        .text().collect::<Vec<_>>().join(" ")
        .trim().to_string();

    // the classroom_id is href of <a class="s">...</a>
    let _classroom_id = single_group_html.select(&s_class_selector).next().unwrap()
        .value().attr("href").unwrap()
        .split('.').next().unwrap()
        .to_string();

    GroupScheduleField {
        subject,
        teacher,
        classroom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_schedule_from_html() {
        let raw_html = include_str!("./test_assets/plany_o27.html");
        let html = Html::parse_document(raw_html);
        
        let schedule = get_schedule_from_html(html);
        
        assert_eq!(schedule.len(), 9);
    }
}