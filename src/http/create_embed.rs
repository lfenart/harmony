use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Default, Serialize)]
pub struct CreateEmbed {
    title: Option<String>,
    // kind: Option<String>,
    description: Option<String>,
    url: Option<String>,
    timestamp: Option<DateTime<Utc>>,
}

impl CreateEmbed {
    pub fn title<T: ToString>(mut self, title: T) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn description<T: ToString>(mut self, description: T) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
}

// title?	string	title of embed
// type?	string	type of embed (always "rich" for webhook embeds)
// description?	string	description of embed
// url?	string	url of embed
// timestamp?	ISO8601 timestamp	timestamp of embed content
// color?	integer	color code of the embed
// footer?	embed footer object	footer information
// image?	embed image object	image information
// thumbnail?	embed thumbnail object	thumbnail information
// video?	embed video object	video information
// provider?	embed provider object	provider information
// author?	embed author object	author information
// fields?	array of embed field objects	fields information
