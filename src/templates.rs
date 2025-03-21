use maud::{ html, Markup, Render, DOCTYPE };

pub fn base(head: Markup, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        head {
            meta name="viewport" content="width=device-width,initial-scale=1.0";
            script src="https://unpkg.com/htmx.org@2.0.2" integrity="sha384-Y7hw+L/jvKeWIRRkqWYfPcvVxHzVzn5REgzbawhxAuQGwX1XWe70vji+VSeHOThJ" crossorigin="anonymous" {}
            title { "CAJHS" }
            (head)
        }
        body {
            (content)
        }
    }
}

pub struct Css(pub &'static str);

impl Render for Css {
    fn render(&self) -> Markup {
        html! {
            style {
                (self.0)
            }
        }
    }
}

pub fn username_validation_template(
    message: &'static str,
    class: &'static str,
    value: String,
    valid: &'static str,
) -> Markup {
    html! {
        div hx-target="this" hx-swap="outerHTML" class="form-div" {
            label { "Send Recommendation to" }
            input required name="to" type="text" placeholder="Username" hx-post="/new/username" value=(value);
            input type="hidden" name="valid_username" value=(valid);
            p class=(class) { (message) }
        }
    }
}
