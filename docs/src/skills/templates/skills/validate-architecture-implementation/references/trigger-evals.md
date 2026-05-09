@filedocuments spec SPECIAL.TRACE_COMMAND.ARCH

# Trigger Evals
@implements SPECIAL.DOCUMENTATION.SKILLS.FALLBACK.VALIDATE_ARCHITECTURE_IMPLEMENTATION
@applies DOCS.SKILL_TRIGGER_EVAL_REFERENCE

## Should Trigger

- Check whether this file really belongs to the export module.
- Review whether this subsystem matches its architecture description.
- This module annotation looks stale after the refactor.
- Find modules with missing or too-broad implementation attachments.
- Inspect this boundary for hidden complexity or outbound-heavy code.

## Should Not Trigger

- Check whether this spec is really supported by its verify.
- Rewrite this product claim.
- Find planned product work.
- Add a recurring implementation pattern.
