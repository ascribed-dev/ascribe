<ascribe-note type="note" label="Note">

A one-line note.

</ascribe-note>

<ascribe-note type="tip" label="Pro tip">

A note on the paragraph after it.

</ascribe-note>

<ascribe-note type="warning" label="Warning" heading="A titled note">

A container note.

With two paragraphs.

</ascribe-note>

<ascribe-steps>

1. Do this.
2. Then this.

</ascribe-steps>

<details>
<summary>More detail</summary>

Hidden until opened.

</details>

<ascribe-availability scope="block">
<ascribe-availability-target target="cloud" dimension="deployment" states="ga">Quill Cloud (GA)</ascribe-availability-target>
</ascribe-availability>

A cloud-only paragraph.

## A section for some <ascribe-attributes id="a-section-for-some"></ascribe-attributes>

<ascribe-availability scope="section">
<ascribe-availability-target target="self-managed" dimension="deployment" states="ga" versions="3.5">Self-managed (GA, 3.5+)</ascribe-availability-target>
</ascribe-availability>

Only some see this.

<quill-labspace lab="first-sync"></quill-labspace>

<quill-aside>

An aside in one line.

</quill-aside>

<quill-aside heading="An aside">

An aside around a paragraph.

</quill-aside>

<ascribe-group widget="quill-compare">

<quill-compare heading="Before" highlight="false">

Old way.

</quill-compare>

<quill-compare heading="After" highlight="false">

New way.

</quill-compare>

</ascribe-group>

<ascribe-tabs sync="pm">

<ascribe-tab value="npm" label="npm">

Use npm.

</ascribe-tab>

<ascribe-tab value="pnpm" label="pnpm">

Use pnpm.

</ascribe-tab>

</ascribe-tabs>
