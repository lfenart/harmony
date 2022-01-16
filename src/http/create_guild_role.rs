use serde::Serialize;

#[derive(Default, Serialize)]
pub struct CreateGuildRole {
    name: Option<String>,
    // permissions:
    color: u32,
    hoist: bool,
    // icon:
    // unicode_emoji:
    mentionable: bool,
}

impl CreateGuildRole {
    pub fn name<T: ToString>(mut self, name: T) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }

    pub fn hoist(mut self, hoist: bool) -> Self {
        self.hoist = hoist;
        self
    }

    pub fn mentionable(mut self, mentionable: bool) -> Self {
        self.mentionable = mentionable;
        self
    }
}

// name	string	name of the role	"new role"
// permissions	string	bitwise value of the enabled/disabled permissions	@everyone permissions in guild
// color	integer	RGB color value	0
// hoist	boolean	whether the role should be displayed separately in the sidebar	false
// icon	image data	the role's icon image (if the guild has the ROLE_ICONS feature)	null
// unicode_emoji	string	the role's unicode emoji as a standard emoji (if the guild has the ROLE_ICONS feature)	null
// mentionable	boolean	whether the role should be mentionable	false
