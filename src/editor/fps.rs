use crate::dom_factory::document;
use std::str::FromStr;

pub fn setup(capped_fps: Option<f64>){
    let height = if let Some(fps) = capped_fps {
        fps * 2.
    } else {
        100.
    };
    let width = height * 2.;
    let dx = width / 100.;
    let viewbox = format!("-{} -{} {} {}", dx.floor(), dx.floor(), (width + 2. * dx).ceil(), (height + 2. * dx).ceil());
    let mut path = format!("M 0 0");
    for i in 0..101 {
        path.push_str(&format!(" L {} 0", i as f64 * dx));
    }
    path.push_str(&format!(" L {} 0 Z", width));
    crate::dom_factory::body()
        .insert_adjacent_html(
            "beforeend",
            maud::html! {
                div#fps {
                    span{}
                    svg viewbox=(viewbox) xmlns="http://www.w3.org/2000/svg" {
                        path d=(path) stroke="#aaa" fill="#222" stroke-width=(dx.to_string()) {} 
                    }
                }
            }
            .into_string()
            .as_str(),
        )
        .expect("Couldn't insert console into the DOM!");
}
pub fn log(fps: f64) {
    if let Some(fps_el) = document().get_element_by_id("fps") {
        let children = fps_el.children();
        let span = children.item(0).unwrap();
        let svg = children.item(1).unwrap();
        let line = svg.children().item(0).unwrap();
        let data = line.get_attribute("d").unwrap();
        let parsed: Vec<&str> = data.split("L").collect();
        let stripped = &parsed[2..102];
        let mut new_data = Vec::new();
        for each in stripped.iter() {
            let each_d: Vec<&str> = each.trim().split(" ").collect();
            new_data.push(f64::from_str(each_d[1]).unwrap());
        }
        new_data.push(f64::floor(fps));
        let avg_fps = new_data.iter().fold(0., |acc, e| acc + e) / new_data.len() as f64;
        let top_fps = new_data.iter().fold(0., |acc, e| if e > &acc { *e } else { acc }) as f64;
        let attr = svg.get_attribute("viewBox").unwrap();
        let attrib: Vec<&str> = attr.split(" ").collect();
        let height = f64::from_str(attrib[3]).unwrap();
        let width = if (top_fps - height).abs() > 1. {
            let height = top_fps;
            let width = height * 2.;
            let dx = width / 100.;
            let viewbox = format!("-{} -{} {} {}", dx.floor(), dx.floor(), (width + 2. * dx).ceil(), (height + 2. * dx).ceil());
            svg.set_attribute("viewBox", &viewbox).unwrap();
            line.set_attribute("stroke-width", &dx.to_string()).unwrap();
            width
        } else {
            100.
        };
        let dx = width / 100.;
        let mut new_attr = format!("M 0 0 L 0 {}", new_data[0]);
        for i in 1..101 {
            new_attr.push_str(&format!(" L {} {}", i as f64 * dx, new_data[i]));
        }
        new_attr.push_str(&format!(" L {} 0 Z", width));
        line.set_attribute("d", &new_attr).unwrap();
        span.set_inner_html(&format!("{:.0}", f64::floor(avg_fps)));
    }
}
