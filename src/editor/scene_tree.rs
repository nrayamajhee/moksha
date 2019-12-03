use super::NodeRef;
use crate::{
    dom_factory::{
        add_class, add_event, body, create_el, create_el_w_class_n_inner, el_innerh, get_el,
        get_parent, get_target_el, get_target_innerh, get_target_parent_el, insert_el,
        insert_el_at, query_el, remove_class,
    },
    log, Editor, Node, RcRcell,
};
use maud::html;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
pub fn build(editor: &Editor) {
    body()
        .insert_adjacent_html("beforeend", markup().as_str())
        .expect("Couldn't insert console into the DOM!");
    build_node(
        editor,
        &get_el("scene-tree"),
        NodeRef::Mutable(editor.scene().root()),
    );
}

fn markup() -> String {
    let markup = html! {
            section #right-panel {
                section #scene-tree.panel {
                    h3 {"Scene Tree"}
                }
                section #properties.panel {
                    h3 {"Properties"}
                }
            }
    };
    markup.into_string()
}

pub fn build_node(editor: &Editor, parent: &Element, node: NodeRef) -> Element {
    let p = create_el("p");
    let li = create_el("li");
    insert_el(&li, &p);
    let ul = create_el("ul");
    let add_collapse_icon = |children: &Vec<RcRcell<Node>>, owned_children: &Vec<Node>| {
        if !children.is_empty() || !owned_children.is_empty() {
            let icon = if owned_children.is_empty() {
                "expand_less"
            } else {
                "expand_more"
            };
            let i = create_el_w_class_n_inner("i", "material-icons fold foldable", icon);
            insert_el(&li, &i);
        } else {
            let i = create_el_w_class_n_inner("i", "material-icons fold", "control_camera");
            insert_el(&li, &i);
        }
    };
    let recurse_children = |children: &Vec<RcRcell<Node>>, owned_children: &Vec<Node>| {
        for child in children {
            let child_el = build_node(editor, &ul, NodeRef::Mutable(child.clone()));
            let li = create_el("li");
            insert_el(&li, &child_el);
            insert_el(&ul, &li);
        }
        for child in owned_children {
            let child_el = build_node(editor, &ul, NodeRef::Owned(child));
            handle_node_folding(&child_el);
            let li = create_el("li");
            insert_el(&li, &child_el);
            insert_el(&ul, &li);
        }
    };
    insert_el(&ul, &li);
    let name = match node {
        NodeRef::Mutable(n) => {
            let node = n.borrow();
            let (children, owned_children) = (node.children(), node.owned_children());
            add_collapse_icon(children, owned_children);
            if parent.id().as_str() != "scene-tree" {
                let eyei = create_el_w_class_n_inner("i", "material-icons eye", "visibility");
                insert_el(&li, &eyei);
                add_node_events(editor, &ul, n.clone());
            } else {
                handle_node_folding(&ul);
            }
            add_drag_events(&p, editor);
            let name = node.info().name;
            add_class(&ul, "shown");
            recurse_children(children, owned_children);
            p.set_attribute("draggable", "true").unwrap();
            name
        }
        NodeRef::Owned(n) => {
            let (children, owned_children) = (n.children(), n.owned_children());
            add_collapse_icon(children, owned_children);
            add_class(&ul, "disabled");
            recurse_children(children, owned_children);
            n.info().name
        }
    };
    let p = p.dyn_into::<HtmlElement>().unwrap();
    p.set_inner_html(name.as_str());
    insert_el(&parent, &ul);
    ul
}
fn get_title_els(el: &Element) -> (Element, Element) {
    let children = el.children().item(0).unwrap().children();
    (children.item(0).unwrap(), children.item(2).unwrap())
}
fn add_node_events(editor: &Editor, el: &Element, node: RcRcell<Node>) {
    let (p, eyei) = get_title_els(el);
    handle_node_folding(&el);
    let a_node = node.clone();
    let a_editor = editor.clone();
    add_event(&p, "click", move |_| {
        a_editor.set_active_node(a_node.clone());
    });
    let a_node = node.clone();
    let scene = editor.scene();
    add_event(&eyei, "click", move |e| {
        match get_target_innerh(&e).as_str() {
            "visibility" => {
                get_target_el(&e).set_inner_html("visibility_off");
                scene.hide_only(&a_node.borrow());
                scene.turn_lights_off(&a_node.borrow());
            }
            "visibility_off" => {
                get_target_el(&e).set_inner_html("visibility");
                scene.show_only(&a_node.borrow());
                scene.turn_lights_on(&a_node.borrow());
            }
            _ => (),
        }
    });
}
fn handle_node_folding(el: &Element) {
    let el = el.children().item(0).unwrap().children().item(1).unwrap();
    if el.class_list().contains("foldable") {
        add_event(&el, "click", move |e| {
            let icon = get_target_innerh(&e);
            let children = get_target_parent_el(&e, 2).children();
            for i in 1..children.length() {
                let class_list = children
                    .item(i)
                    .unwrap()
                    .children()
                    .item(0)
                    .unwrap()
                    .class_list();
                match icon.as_str() {
                    "expand_more" => {
                        class_list.add_1("shown").unwrap();
                    }
                    "expand_less" => {
                        class_list.remove_1("shown").unwrap();
                    }
                    _ => (),
                }
            }
            let next_icon = match icon.as_str() {
                "expand_more" => "expand_less",
                _ => "expand_more",
            };
            get_target_el(&e).set_inner_html(next_icon);
        });
    }
}
fn add_drag_events(el: &Element, editor: &Editor) {
    add_event(el, "dragenter", move |e| {
        add_class(&get_target_el(&e), "dragenter");
    });
    add_event(el, "dragleave", move |e| {
        remove_class(&get_target_el(&e), "dragenter");
    });
    add_event(el, "dragstart", move |e| {
        add_class(&get_target_el(&e), "dragged-el");
    });
    add_event(el, "dragend", move |e| {
        remove_class(&get_target_el(&e), "dragged-el");
    });
    add_event(el, "dragover", |e| {
        e.prevent_default();
    });
    let editor = editor.clone();
    add_event(el, "drop", move |e| {
        remove_class(&get_target_el(&e), "dragenter");
        let dragged_el = query_el("#scene-tree p.dragged-el");
        let dragged_el_name = el_innerh(dragged_el.clone());
        let dragged_parent_el = get_parent(&dragged_el, 4).unwrap();
        let dragged_parent_name = el_innerh(
            dragged_parent_el
                .children()
                .item(0)
                .unwrap()
                .children()
                .item(0)
                .unwrap(),
        );
        log!(dragged_parent_name);
        let drop_target_name = get_target_innerh(&e);
        if drop_target_name != dragged_el_name && drop_target_name != dragged_parent_name {
            log!("Dropping");
            log!(dragged_el_name dragged_parent_name drop_target_name);
            let scene = editor.scene();
            let dragged_node = scene.find_node_w_name(&dragged_el_name).unwrap();
            let parent_node = scene.find_node_w_name(&dragged_parent_name).unwrap();
            let target_node = scene.find_node_w_name(&drop_target_name).unwrap();
            parent_node.borrow_mut().remove(&dragged_el_name);
            target_node.borrow_mut().add(dragged_node.clone());
            if dragged_parent_name.as_str() != "Scene" {
                let li = create_el("li");
                let g_p_el = get_parent(&dragged_el, 6).unwrap();
                build_node(&editor, &li, NodeRef::Mutable(parent_node));
                insert_el(&g_p_el, &li);
            } else {
                // this is direct children of root
                build_node(
                    &editor,
                    &get_parent(&dragged_el, 5).unwrap(),
                    NodeRef::Mutable(parent_node),
                );
            }
            let li = create_el("li");
            build_node(&editor, &li, NodeRef::Mutable(dragged_node));
            insert_el_at(&get_target_parent_el(&e, 1), &li, "afterend");
            get_parent(&dragged_el, 4).unwrap().remove();
        }
    });
}
