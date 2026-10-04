// A stylesheet's text, as Vite imports it with `?inline`.
declare module "*.css?inline" {
  const css: string;
  export default css;
}
