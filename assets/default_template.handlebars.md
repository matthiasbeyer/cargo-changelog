# CHANGELOG

<!-- generated from cargo-changelog -->

{{#if this.versions}}
{{#each (reverse (sort_versions this.versions))}}
## v{{this.version}}

{{#each this.entries}}
- (#{{this.header.issue}}) {{this.text}}
{{/each}}

{{/each}}
{{/if}}
