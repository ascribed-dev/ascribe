@note {type = tip
Unclosed block.

@note { bare }: A bare key.

@note {type = tip, type = caution}: A repeated key.

@quill-labspace {lab=Using other images  ,  height=3}

@steps   foo
1. One.

@include :  my file.md

@steps :   text
1. One.

@variant   {pm=npm}
Container-only without its colon.
@end

![Alt text](images/a.png){ width = 3, width = 4 }

@note {type=caution}: Formatted.
