# Roadmap

- [ ] **Session-title mode.** The engine today names projects: prefixes,
      `composed`/`coined` strategies, embedded lexicon. Add a mode that turns
      a short content (a first prompt, an objective) into a descriptive
      *title* — still deterministic, offline and multilanguage — with no
      prefix required. The consumer is `actsessions`, which renames agent
      sessions from their content and would take Actname by git tag.
      *Split of responsibilities (2026-11-13):* Actname derives the title
      from the content; actsessions owns the rename act and the session
      store. Dependency direction: `actsessions → Actname`, never the
      reverse — Actname stays a pure engine that does not know what a
      session is. *Accepted when:* the same content always yields the same
      title (pinned by tests), and the mode works from `--domain` and from
      a `--readme`/context file like the existing strategies.
