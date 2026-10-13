A tested line alone:

```ts tested="Node 24.9.0 on Linux"
console.log("hello");
```

A title and a tested line:

```ts title="hello.ts" tested="Node 24.9.0 and TypeScript 5.9.2 on Linux"
console.log("hello");
```

A tested line that looks like markup:

```ts tested="<script>alert(1)</script> 1.0 on Linux"
console.log("hello");
```

No info past the language:

```ts
console.log("hello");
```
