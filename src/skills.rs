/**
@module SPECIAL.SKILLS
Bundled skill catalog and install-facing asset definitions in `src/skills.rs`. This module owns which bundled skills ship with the binary and the files each skill installation writes.

@spec SPECIAL.SKILLS.BUNDLED_FRONTMATTER_VALIDATION
special validates bundled `SKILL.md` frontmatter before printing or installing bundled skills. Bundled skill frontmatter must declare a matching `name` and a quoted `description`.
*/
// @fileimplements SPECIAL.SKILLS
mod install;

use anyhow::Result;

pub(crate) use install::{
    conflicting_skill_paths, install_bundled_skills, resolve_global_skills_root,
};

use anyhow::{Context, bail};

pub struct SkillAsset {
    relative_path: &'static str,
    contents: &'static str,
}

pub struct BundledSkill {
    pub id: &'static str,
    pub summary: &'static str,
    assets: &'static [SkillAsset],
}

const SHIP_PRODUCT_CHANGE_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/ship-product-change/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/change-workflow.md",
        contents: include_str!(
            "../templates/skills/ship-product-change/references/change-workflow.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/ship-product-change/references/trigger-evals.md"
        ),
    },
];

const DEFINE_PRODUCT_SPECS_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/define-product-specs/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/spec-writing.md",
        contents: include_str!(
            "../templates/skills/define-product-specs/references/spec-writing.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/define-product-specs/references/trigger-evals.md"
        ),
    },
];

const VALIDATE_PRODUCT_CONTRACT_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/validate-product-contract/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/validation-checklist.md",
        contents: include_str!(
            "../templates/skills/validate-product-contract/references/validation-checklist.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/validate-product-contract/references/trigger-evals.md"
        ),
    },
];

const VALIDATE_ARCHITECTURE_IMPLEMENTATION_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/validate-architecture-implementation/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/validation-checklist.md",
        contents: include_str!(
            "../templates/skills/validate-architecture-implementation/references/validation-checklist.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/validate-architecture-implementation/references/trigger-evals.md"
        ),
    },
];

const USE_PROJECT_PATTERNS_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/use-project-patterns/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/pattern-workflow.md",
        contents: include_str!(
            "../templates/skills/use-project-patterns/references/pattern-workflow.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/use-project-patterns/references/trigger-evals.md"
        ),
    },
];

const EVOLVE_MODULE_ARCHITECTURE_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/evolve-module-architecture/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/evolve-module-architecture/references/trigger-evals.md"
        ),
    },
];

const INSPECT_CURRENT_SPEC_STATE_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/inspect-current-spec-state/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/state-walkthrough.md",
        contents: include_str!(
            "../templates/skills/inspect-current-spec-state/references/state-walkthrough.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!(
            "../templates/skills/inspect-current-spec-state/references/trigger-evals.md"
        ),
    },
];

const FIND_PLANNED_WORK_ASSETS: &[SkillAsset] = &[
    SkillAsset {
        relative_path: "SKILL.md",
        contents: include_str!("../templates/skills/find-planned-work/SKILL.md"),
    },
    SkillAsset {
        relative_path: "references/planned-workflow.md",
        contents: include_str!(
            "../templates/skills/find-planned-work/references/planned-workflow.md"
        ),
    },
    SkillAsset {
        relative_path: "references/trigger-evals.md",
        contents: include_str!("../templates/skills/find-planned-work/references/trigger-evals.md"),
    },
];

const SETUP_SPECIAL_PROJECT_ASSETS: &[SkillAsset] = &[SkillAsset {
    relative_path: "SKILL.md",
    contents: include_str!("../templates/skills/setup-special-project/SKILL.md"),
}];

const INSTALL_OR_UPDATE_SPECIAL_ASSETS: &[SkillAsset] = &[SkillAsset {
    relative_path: "SKILL.md",
    contents: include_str!("../templates/skills/install-or-update-special/SKILL.md"),
}];

const SPECIAL_WORKFLOW_ASSETS: &[SkillAsset] = &[SkillAsset {
    relative_path: "SKILL.md",
    contents: include_str!("../templates/skills/special-workflow/SKILL.md"),
}];

const WRITE_SPECIAL_DOCS_ASSETS: &[SkillAsset] = &[SkillAsset {
    relative_path: "SKILL.md",
    contents: include_str!("../templates/skills/write-special-docs/SKILL.md"),
}];

const AUDIT_DOCS_RELATIONSHIPS_ASSETS: &[SkillAsset] = &[SkillAsset {
    relative_path: "SKILL.md",
    contents: include_str!("../templates/skills/audit-docs-relationships/SKILL.md"),
}];

const REVIEW_SPECIAL_TRACE_ASSETS: &[SkillAsset] = &[SkillAsset {
    relative_path: "SKILL.md",
    contents: include_str!("../templates/skills/review-special-trace/SKILL.md"),
}];

const INTERPRET_SPECIAL_HEALTH_ASSETS: &[SkillAsset] = &[SkillAsset {
    relative_path: "SKILL.md",
    contents: include_str!("../templates/skills/interpret-special-health/SKILL.md"),
}];

const BUNDLED_SKILLS: &[BundledSkill] = &[
    BundledSkill {
        id: "ship-product-change",
        summary: "Ship a product change without changing the contract by accident.",
        assets: SHIP_PRODUCT_CHANGE_ASSETS,
    },
    BundledSkill {
        id: "define-product-specs",
        summary: "Turn requirements into explicit product specs.",
        assets: DEFINE_PRODUCT_SPECS_ASSETS,
    },
    BundledSkill {
        id: "validate-product-contract",
        summary: "Check whether one claim is honestly supported.",
        assets: VALIDATE_PRODUCT_CONTRACT_ASSETS,
    },
    BundledSkill {
        id: "validate-architecture-implementation",
        summary: "Check whether one module is honestly implemented.",
        assets: VALIDATE_ARCHITECTURE_IMPLEMENTATION_ASSETS,
    },
    BundledSkill {
        id: "use-project-patterns",
        summary: "Follow, define, and review adopted implementation patterns.",
        assets: USE_PROJECT_PATTERNS_ASSETS,
    },
    BundledSkill {
        id: "evolve-module-architecture",
        summary: "Update tracked architecture intent in the real module tree.",
        assets: EVOLVE_MODULE_ARCHITECTURE_ASSETS,
    },
    BundledSkill {
        id: "inspect-current-spec-state",
        summary: "Inspect the current validated spec state.",
        assets: INSPECT_CURRENT_SPEC_STATE_ASSETS,
    },
    BundledSkill {
        id: "find-planned-work",
        summary: "Find planned product-spec work that is not current yet.",
        assets: FIND_PLANNED_WORK_ASSETS,
    },
    BundledSkill {
        id: "setup-special-project",
        summary: "Configure and validate Special in a project.",
        assets: SETUP_SPECIAL_PROJECT_ASSETS,
    },
    BundledSkill {
        id: "install-or-update-special",
        summary: "Install or update the Special binary and Codex plugin.",
        assets: INSTALL_OR_UPDATE_SPECIAL_ASSETS,
    },
    BundledSkill {
        id: "special-workflow",
        summary: "Use Special surfaces with MCP or CLI fallback.",
        assets: SPECIAL_WORKFLOW_ASSETS,
    },
    BundledSkill {
        id: "write-special-docs",
        summary: "Author generated docs source with traceable claims.",
        assets: WRITE_SPECIAL_DOCS_ASSETS,
    },
    BundledSkill {
        id: "audit-docs-relationships",
        summary: "Review docs claims against linked Special targets.",
        assets: AUDIT_DOCS_RELATIONSHIPS_ASSETS,
    },
    BundledSkill {
        id: "review-special-trace",
        summary: "Use trace packets for focused alignment review.",
        assets: REVIEW_SPECIAL_TRACE_ASSETS,
    },
    BundledSkill {
        id: "interpret-special-health",
        summary: "Turn health output into scoped follow-up work.",
        assets: INTERPRET_SPECIAL_HEALTH_ASSETS,
    },
];

pub fn bundled_skills() -> &'static [BundledSkill] {
    BUNDLED_SKILLS
}

pub fn bundled_skill(skill_id: &str) -> Option<&'static BundledSkill> {
    BUNDLED_SKILLS.iter().find(|skill| skill.id == skill_id)
}

pub(crate) fn primary_skill_contents(skill_id: &str) -> Result<&'static str> {
    let skill =
        bundled_skill(skill_id).ok_or_else(|| anyhow::anyhow!("unknown skill id `{skill_id}`"))?;
    validate_bundled_skill_frontmatter(skill)?;
    skill
        .assets
        .iter()
        .find(|asset| asset.relative_path == "SKILL.md")
        .map(|asset| asset.contents)
        .ok_or_else(|| anyhow::anyhow!("bundled skill `{skill_id}` is missing `SKILL.md`"))
}

pub(crate) fn validate_bundled_skill_frontmatter(skill: &BundledSkill) -> Result<()> {
    let contents = skill
        .assets
        .iter()
        .find(|asset| asset.relative_path == "SKILL.md")
        .map(|asset| asset.contents)
        .ok_or_else(|| anyhow::anyhow!("bundled skill `{}` is missing `SKILL.md`", skill.id))?;
    let frontmatter = parse_skill_frontmatter(contents).with_context(|| {
        format!(
            "bundled skill `{}` has invalid SKILL.md frontmatter",
            skill.id
        )
    })?;
    if frontmatter.name.as_deref() != Some(skill.id) {
        bail!(
            "bundled skill `{}` has invalid SKILL.md frontmatter: name must be `{}`",
            skill.id,
            skill.id
        );
    }
    if frontmatter.description.is_none() {
        bail!(
            "bundled skill `{}` has invalid SKILL.md frontmatter: missing description",
            skill.id
        );
    }
    Ok(())
}

#[derive(Debug, Default)]
struct SkillFrontmatter {
    name: Option<String>,
    description: Option<String>,
}

fn parse_skill_frontmatter(contents: &str) -> Result<SkillFrontmatter> {
    let mut lines = contents.lines();
    if lines.next() != Some("---") {
        bail!("SKILL.md must start with YAML frontmatter");
    }

    let mut frontmatter = SkillFrontmatter::default();
    for (index, line) in lines.enumerate() {
        let line_number = index + 2;
        if line == "---" {
            return Ok(frontmatter);
        }
        if line.trim().is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            bail!("frontmatter line {line_number} must use `key: value`");
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "name" => {
                frontmatter.name = Some(parse_frontmatter_scalar(value, line_number, false)?);
            }
            "description" => {
                frontmatter.description = Some(parse_frontmatter_scalar(value, line_number, true)?);
            }
            _ => {}
        }
    }

    bail!("SKILL.md frontmatter must close with `---`")
}

fn parse_frontmatter_scalar(
    value: &str,
    line_number: usize,
    require_quoted: bool,
) -> Result<String> {
    if value.is_empty() {
        bail!("frontmatter line {line_number} must not use an empty value");
    }

    let quoted = parse_quoted_frontmatter_scalar(value)?;
    match (quoted, require_quoted) {
        (Some(value), _) => Ok(value),
        (None, true) => bail!("frontmatter line {line_number} description must be quoted"),
        (None, false) => Ok(value.to_string()),
    }
}

fn parse_quoted_frontmatter_scalar(value: &str) -> Result<Option<String>> {
    let Some(quote) = value
        .chars()
        .next()
        .filter(|quote| *quote == '\'' || *quote == '"')
    else {
        return Ok(None);
    };
    if !value.ends_with(quote) || value.len() == 1 {
        bail!("quoted frontmatter value must end with a matching quote");
    }
    let inner = &value[1..value.len() - 1];
    if quote == '\'' {
        let mut parsed = String::new();
        let mut chars = inner.chars().peekable();
        while let Some(character) = chars.next() {
            if character == '\'' {
                if chars.peek() == Some(&'\'') {
                    chars.next();
                    parsed.push('\'');
                } else {
                    bail!("single-quoted frontmatter values must escape `'` as `''`");
                }
            } else {
                parsed.push(character);
            }
        }
        Ok(Some(parsed))
    } else {
        let mut parsed = String::new();
        let mut chars = inner.chars();
        while let Some(character) = chars.next() {
            if character == '"' {
                bail!("double-quoted frontmatter values must escape `\"` as `\\\"`");
            }
            if character == '\\' {
                match chars.next() {
                    Some('"') => parsed.push('"'),
                    Some('\\') => parsed.push('\\'),
                    Some(other) => {
                        parsed.push('\\');
                        parsed.push(other);
                    }
                    None => bail!("double-quoted frontmatter value must not end in `\\`"),
                }
            } else {
                parsed.push(character);
            }
        }
        Ok(Some(parsed))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BundledSkill, SkillAsset, parse_skill_frontmatter, validate_bundled_skill_frontmatter,
    };

    #[test]
    // @verifies SPECIAL.SKILLS.BUNDLED_FRONTMATTER_VALIDATION
    fn skill_frontmatter_requires_quoted_description() {
        let error = parse_skill_frontmatter(
            "---\nname: example\ndescription: Use this when values contain: punctuation\n---\n",
        )
        .expect_err("unquoted description should fail");

        assert!(error.to_string().contains("description must be quoted"));
    }

    #[test]
    // @verifies SPECIAL.SKILLS.BUNDLED_FRONTMATTER_VALIDATION
    fn skill_frontmatter_accepts_quoted_description_with_punctuation() {
        let parsed = parse_skill_frontmatter(
            "---\nname: example\ndescription: 'Use this when values contain: punctuation and `code`.'\n---\n",
        )
        .expect("quoted description should parse");

        assert_eq!(parsed.name.as_deref(), Some("example"));
        assert_eq!(
            parsed.description.as_deref(),
            Some("Use this when values contain: punctuation and `code`.")
        );
    }

    #[test]
    // @verifies SPECIAL.SKILLS.BUNDLED_FRONTMATTER_VALIDATION
    fn skill_frontmatter_accepts_nested_double_quotes_in_single_quoted_description() {
        let parsed = parse_skill_frontmatter(
            "---\nname: example\ndescription: 'Capture the \"why is it done this way?\" answer.'\n---\n",
        )
        .expect("single-quoted description should allow double quotes");

        assert_eq!(
            parsed.description.as_deref(),
            Some("Capture the \"why is it done this way?\" answer.")
        );
    }

    #[test]
    // @verifies SPECIAL.SKILLS.BUNDLED_FRONTMATTER_VALIDATION
    fn skill_frontmatter_rejects_missing_frontmatter() {
        let error =
            parse_skill_frontmatter("# Example\n").expect_err("frontmatter should be required");

        assert!(
            error
                .to_string()
                .contains("must start with YAML frontmatter")
        );
    }

    #[test]
    // @verifies SPECIAL.SKILLS.BUNDLED_FRONTMATTER_VALIDATION
    fn skill_frontmatter_rejects_unclosed_frontmatter() {
        let error = parse_skill_frontmatter("---\nname: example\ndescription: 'Example.'\n")
            .expect_err("frontmatter should require a closing marker");

        assert!(error.to_string().contains("must close with `---`"));
    }

    #[test]
    // @verifies SPECIAL.SKILLS.BUNDLED_FRONTMATTER_VALIDATION
    fn bundled_skill_frontmatter_requires_matching_name() {
        let skill = BundledSkill {
            id: "expected",
            summary: "Example skill.",
            assets: &[SkillAsset {
                relative_path: "SKILL.md",
                contents: "---\nname: actual\ndescription: 'Example.'\n---\n",
            }],
        };
        let error = validate_bundled_skill_frontmatter(&skill)
            .expect_err("skill name should match bundled skill id");

        assert!(error.to_string().contains("name must be `expected`"));
    }

    #[test]
    // @verifies SPECIAL.SKILLS.BUNDLED_FRONTMATTER_VALIDATION
    fn bundled_skill_frontmatter_requires_description() {
        let skill = BundledSkill {
            id: "example",
            summary: "Example skill.",
            assets: &[SkillAsset {
                relative_path: "SKILL.md",
                contents: "---\nname: example\n---\n",
            }],
        };
        let error = validate_bundled_skill_frontmatter(&skill)
            .expect_err("skill description should be required");

        assert!(error.to_string().contains("missing description"));
    }
}
