# AO Data Tool Suite

A collection of tools for managing Argentum Online game data.

## Why This Exists

Argentum Online is an MMORPG developed in 1999 that stores all game data in text files (.CHR for characters, .DAT for objects/maps/NPCs/spells) instead of a traditional database. This creates several challenges, for example querying and analytics.

This project reads those game data files, imports them to PostgreSQL for analytics, reporting, and other external tools while leaving the game servers unchanged.

Server owners can now generate user rankings, character profiles, NPC & item data with current stats/drops (and more) for their websites, even without migrating their legacy source code to a modern database.

## About Argentum Online

The Argentum Online community has created several open-source rewrites. But the original VB6 implementation and the most popular rewrite are:

- **Alkon 13.3** (Visual Basic 6) - Original implementation. [Server](https://github.com/DakaraOnline/morgoao-server-vb6) - [Client](https://github.com/DakaraOnline/morgoao-client-vb6) - [SourceForge](https://sourceforge.net/projects/morgoao) - [General Forum Archive](https://web.archive.org/web/20100430205238/http://www.alkon.com.ar/foro/argentum-online.53/738002-nueva-version-nueva-cara.html) - [Developer Forum Archive 1](https://web.archive.org/web/20071222193918/http://morgoao.sourceforge.net/smf/index.php/topic,14.0.html) - [Developer Forum Archive 2](https://web.archive.org/web/20080407163400/http://morgoao.sourceforge.net/smf/index.php?topic=187.msg1902#new)
- **Dakara 13.3** (C++) - Modern server rewrite. [Server](https://github.com/DakaraOnline/dakara-server) - [Forum Archive](https://web.archive.org/web/20160811083006/http://foros.dakaraonline.org/)

This toolset supports both versions and their community forks.

## Crates

This is a Rust workspace with multiple crates. If you're familiar with npm/pnpm workspaces in the TypeScript ecosystem, the concept is similar, which is Rust's approach to building a monorepo.

```
ao-data-tool-suite/
├── ao_data_to_sql/          # Imports game data files → PostgreSQL
├── ao_sql_to_static_files/  # Exports PostgreSQL → static JSON files
└── ao_shared/               # Shared utilities (DB connection, etc.)
```

## Running

Each crate has its own README with specific usage instructions and commands. See the individual crate directories for details.

Configuration is handled via `.env` file or environment variables. See `.env.example` for available options.

Docker containerization ensures compatibility with both Windows Server (Alkon-based VB6 legacy servers) and Linux distros (Dakara-based C++ servers).
