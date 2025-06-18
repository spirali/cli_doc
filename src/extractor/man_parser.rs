/*use anyhow::bail;
use crate::commands::{CommandDoc, CommandOuterDoc};
use crate::extractor::sections::Section;
use crate::text::RichText;

pub(crate) struct ManParser {}

impl ManParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(
        &self,
        sections: &mut [Section],
    ) -> anyhow::Result<(CommandDoc, Vec<CommandOuterDoc>)> {
        let Some(section) = sections.get(1) else {
            bail!("2nd section not found");
        };
        if section.first_line() != "NAME" {
            bail!("2nd section is not NAME")
        }
        let brief = if let Some((left, _right)) = section.subsections().first().and_then(|s| s.first_line().split_once(": ")) {
            RichText::from_single_line(left)
        } else {
            bail!("Invalid NAME format")
        };

        Ok(
            (CommandDoc {
                brief,
                description: None,
                usage: vec![],
                arguments: vec![],
                option_categories: vec![],
            }, Vec::new()
            )
        )
        todo!()
    }
}
*/
