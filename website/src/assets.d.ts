declare module '*.css' {}
declare module '*.glb?url' {
  const url: string;
  export default url;
}
declare module '*.glb' {
  const url: string;
  export default url;
}
declare const __UI_BUILD__: string;
