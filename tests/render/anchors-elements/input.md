<ascribe-note type="note" label="Note" data-ascribe-source="index.md:5-5">

<!--ascribe-anchor tag="p" source="index.md:5-5"-->
A one-line note.

</ascribe-note>

<ascribe-note type="tip" label="Pro tip" data-ascribe-source="index.md:7-8">

<!--ascribe-anchor tag="p" source="index.md:8-8"-->
A note on the paragraph after it.

</ascribe-note>

<ascribe-note type="warning" label="Warning" heading="A titled note" data-ascribe-source="index.md:10-15">

<!--ascribe-anchor tag="p" source="index.md:12-12"-->
A container note.

<!--ascribe-anchor tag="p" source="index.md:14-14"-->
With two paragraphs.

</ascribe-note>

<ascribe-steps data-ascribe-source="index.md:17-19">

<!--ascribe-anchor tag="ol" source="index.md:18-19" items="18-18 19-19"-->
1. Do this.
2. Then this.

</ascribe-steps>

<details data-ascribe-source="index.md:21-24">
<summary>More detail</summary>

<!--ascribe-anchor tag="p" source="index.md:23-23"-->
Hidden until opened.

</details>

<ascribe-availability scope="block" data-ascribe-source="index.md:26-26">
<ascribe-availability-target target="cloud" dimension="deployment" states="ga">Quill Cloud (GA)</ascribe-availability-target>
</ascribe-availability>

<!--ascribe-anchor tag="p" source="index.md:27-27"-->
A cloud-only paragraph.

<!--ascribe-anchor tag="h2" source="index.md:29-29"-->
## A section for some <ascribe-attributes id="a-section-for-some"></ascribe-attributes>

<ascribe-availability scope="section" data-ascribe-source="index.md:30-30">
<ascribe-availability-target target="self-managed" dimension="deployment" states="ga" versions="3.5">Self-managed (GA, 3.5+)</ascribe-availability-target>
</ascribe-availability>

<!--ascribe-anchor tag="p" source="index.md:32-32"-->
Only some see this.

<!--ascribe-anchor tag="p" source="index.md:34-34"-->
<quill-labspace lab="first-sync" data-ascribe-source="index.md:34-34"></quill-labspace>

<quill-aside data-ascribe-source="index.md:36-36">

<!--ascribe-anchor tag="p" source="index.md:36-36"-->
An aside in one line.

</quill-aside>

<quill-aside heading="An aside" data-ascribe-source="index.md:38-41">

<!--ascribe-anchor tag="p" source="index.md:40-40"-->
An aside around a paragraph.

</quill-aside>

<ascribe-group widget="quill-compare" data-ascribe-source="index.md:43-50">

<quill-compare heading="Before" highlight="false" data-ascribe-source="index.md:43-45">

<!--ascribe-anchor tag="p" source="index.md:45-45"-->
Old way.

</quill-compare>

<quill-compare heading="After" highlight="false" data-ascribe-source="index.md:47-49">

<!--ascribe-anchor tag="p" source="index.md:49-49"-->
New way.

</quill-compare>

</ascribe-group>

<ascribe-tabs sync="pm" data-ascribe-source="index.md:52-56">

<ascribe-tab value="npm" label="npm" data-ascribe-source="index.md:52-53">

<!--ascribe-anchor tag="p" source="index.md:53-53"-->
Use npm.

</ascribe-tab>

<ascribe-tab value="pnpm" label="pnpm" data-ascribe-source="index.md:54-55">

<!--ascribe-anchor tag="p" source="index.md:55-55"-->
Use pnpm.

</ascribe-tab>

</ascribe-tabs>
