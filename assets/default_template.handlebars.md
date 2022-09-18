# CHANGELOG

<!-- generated from cargo-changelog -->

{{#if this.versions}}
{{#each (reverse (sort_versions this.versions))}}
## v{{this.version}}

{{#each (group_by_header this.entries "type")}}
### {{ @key }}

{{#each this ~}}
#### {{~ #if this.header.issue ~}} (#{{this.header.issue}}){{/if}} {{this.header.subject}}
{{this.text}}
{{/each ~}}
{{~ /each ~}}
{{~ /each ~}}
{{/if}}
