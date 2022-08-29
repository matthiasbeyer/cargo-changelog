# CHANGELOG

<!-- generated from cargo-changelog -->

{{#each (sort_versions this.versions)}}
## v{{this.version}}

{{#each this.entries}}
- (#{{this.header.issue}}) {{this.text}}
{{/each}}

{{/each}}
