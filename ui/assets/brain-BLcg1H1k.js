var uy=Object.defineProperty;var dy=(n,e,t)=>e in n?uy(n,e,{enumerable:!0,configurable:!0,writable:!0,value:t}):n[e]=t;var ue=(n,e,t)=>dy(n,typeof e!="symbol"?e+"":e,t);(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const r of document.querySelectorAll('link[rel="modulepreload"]'))i(r);new MutationObserver(r=>{for(const s of r)if(s.type==="childList")for(const o of s.addedNodes)o.tagName==="LINK"&&o.rel==="modulepreload"&&i(o)}).observe(document,{childList:!0,subtree:!0});function t(r){const s={};return r.integrity&&(s.integrity=r.integrity),r.referrerPolicy&&(s.referrerPolicy=r.referrerPolicy),r.crossOrigin==="use-credentials"?s.credentials="include":r.crossOrigin==="anonymous"?s.credentials="omit":s.credentials="same-origin",s}function i(r){if(r.ep)return;r.ep=!0;const s=t(r);fetch(r.href,s)}})();var r_={exports:{}},Tc={},s_={exports:{}},Ge={};/**
 * @license React
 * react.production.min.js
 *
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */var va=Symbol.for("react.element"),fy=Symbol.for("react.portal"),hy=Symbol.for("react.fragment"),py=Symbol.for("react.strict_mode"),my=Symbol.for("react.profiler"),gy=Symbol.for("react.provider"),_y=Symbol.for("react.context"),vy=Symbol.for("react.forward_ref"),yy=Symbol.for("react.suspense"),xy=Symbol.for("react.memo"),Sy=Symbol.for("react.lazy"),hp=Symbol.iterator;function My(n){return n===null||typeof n!="object"?null:(n=hp&&n[hp]||n["@@iterator"],typeof n=="function"?n:null)}var o_={isMounted:function(){return!1},enqueueForceUpdate:function(){},enqueueReplaceState:function(){},enqueueSetState:function(){}},a_=Object.assign,l_={};function no(n,e,t){this.props=n,this.context=e,this.refs=l_,this.updater=t||o_}no.prototype.isReactComponent={};no.prototype.setState=function(n,e){if(typeof n!="object"&&typeof n!="function"&&n!=null)throw Error("setState(...): takes an object of state variables to update or a function which returns an object of state variables.");this.updater.enqueueSetState(this,n,e,"setState")};no.prototype.forceUpdate=function(n){this.updater.enqueueForceUpdate(this,n,"forceUpdate")};function c_(){}c_.prototype=no.prototype;function Vf(n,e,t){this.props=n,this.context=e,this.refs=l_,this.updater=t||o_}var Gf=Vf.prototype=new c_;Gf.constructor=Vf;a_(Gf,no.prototype);Gf.isPureReactComponent=!0;var pp=Array.isArray,u_=Object.prototype.hasOwnProperty,Wf={current:null},d_={key:!0,ref:!0,__self:!0,__source:!0};function f_(n,e,t){var i,r={},s=null,o=null;if(e!=null)for(i in e.ref!==void 0&&(o=e.ref),e.key!==void 0&&(s=""+e.key),e)u_.call(e,i)&&!d_.hasOwnProperty(i)&&(r[i]=e[i]);var a=arguments.length-2;if(a===1)r.children=t;else if(1<a){for(var l=Array(a),c=0;c<a;c++)l[c]=arguments[c+2];r.children=l}if(n&&n.defaultProps)for(i in a=n.defaultProps,a)r[i]===void 0&&(r[i]=a[i]);return{$$typeof:va,type:n,key:s,ref:o,props:r,_owner:Wf.current}}function Ey(n,e){return{$$typeof:va,type:n.type,key:e,ref:n.ref,props:n.props,_owner:n._owner}}function Xf(n){return typeof n=="object"&&n!==null&&n.$$typeof===va}function Ty(n){var e={"=":"=0",":":"=2"};return"$"+n.replace(/[=:]/g,function(t){return e[t]})}var mp=/\/+/g;function qc(n,e){return typeof n=="object"&&n!==null&&n.key!=null?Ty(""+n.key):e.toString(36)}function Al(n,e,t,i,r){var s=typeof n;(s==="undefined"||s==="boolean")&&(n=null);var o=!1;if(n===null)o=!0;else switch(s){case"string":case"number":o=!0;break;case"object":switch(n.$$typeof){case va:case fy:o=!0}}if(o)return o=n,r=r(o),n=i===""?"."+qc(o,0):i,pp(r)?(t="",n!=null&&(t=n.replace(mp,"$&/")+"/"),Al(r,e,t,"",function(c){return c})):r!=null&&(Xf(r)&&(r=Ey(r,t+(!r.key||o&&o.key===r.key?"":(""+r.key).replace(mp,"$&/")+"/")+n)),e.push(r)),1;if(o=0,i=i===""?".":i+":",pp(n))for(var a=0;a<n.length;a++){s=n[a];var l=i+qc(s,a);o+=Al(s,e,t,l,r)}else if(l=My(n),typeof l=="function")for(n=l.call(n),a=0;!(s=n.next()).done;)s=s.value,l=i+qc(s,a++),o+=Al(s,e,t,l,r);else if(s==="object")throw e=String(n),Error("Objects are not valid as a React child (found: "+(e==="[object Object]"?"object with keys {"+Object.keys(n).join(", ")+"}":e)+"). If you meant to render a collection of children, use an array instead.");return o}function Pa(n,e,t){if(n==null)return n;var i=[],r=0;return Al(n,i,"","",function(s){return e.call(t,s,r++)}),i}function wy(n){if(n._status===-1){var e=n._result;e=e(),e.then(function(t){(n._status===0||n._status===-1)&&(n._status=1,n._result=t)},function(t){(n._status===0||n._status===-1)&&(n._status=2,n._result=t)}),n._status===-1&&(n._status=0,n._result=e)}if(n._status===1)return n._result.default;throw n._result}var rn={current:null},Rl={transition:null},Ay={ReactCurrentDispatcher:rn,ReactCurrentBatchConfig:Rl,ReactCurrentOwner:Wf};function h_(){throw Error("act(...) is not supported in production builds of React.")}Ge.Children={map:Pa,forEach:function(n,e,t){Pa(n,function(){e.apply(this,arguments)},t)},count:function(n){var e=0;return Pa(n,function(){e++}),e},toArray:function(n){return Pa(n,function(e){return e})||[]},only:function(n){if(!Xf(n))throw Error("React.Children.only expected to receive a single React element child.");return n}};Ge.Component=no;Ge.Fragment=hy;Ge.Profiler=my;Ge.PureComponent=Vf;Ge.StrictMode=py;Ge.Suspense=yy;Ge.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED=Ay;Ge.act=h_;Ge.cloneElement=function(n,e,t){if(n==null)throw Error("React.cloneElement(...): The argument must be a React element, but you passed "+n+".");var i=a_({},n.props),r=n.key,s=n.ref,o=n._owner;if(e!=null){if(e.ref!==void 0&&(s=e.ref,o=Wf.current),e.key!==void 0&&(r=""+e.key),n.type&&n.type.defaultProps)var a=n.type.defaultProps;for(l in e)u_.call(e,l)&&!d_.hasOwnProperty(l)&&(i[l]=e[l]===void 0&&a!==void 0?a[l]:e[l])}var l=arguments.length-2;if(l===1)i.children=t;else if(1<l){a=Array(l);for(var c=0;c<l;c++)a[c]=arguments[c+2];i.children=a}return{$$typeof:va,type:n.type,key:r,ref:s,props:i,_owner:o}};Ge.createContext=function(n){return n={$$typeof:_y,_currentValue:n,_currentValue2:n,_threadCount:0,Provider:null,Consumer:null,_defaultValue:null,_globalName:null},n.Provider={$$typeof:gy,_context:n},n.Consumer=n};Ge.createElement=f_;Ge.createFactory=function(n){var e=f_.bind(null,n);return e.type=n,e};Ge.createRef=function(){return{current:null}};Ge.forwardRef=function(n){return{$$typeof:vy,render:n}};Ge.isValidElement=Xf;Ge.lazy=function(n){return{$$typeof:Sy,_payload:{_status:-1,_result:n},_init:wy}};Ge.memo=function(n,e){return{$$typeof:xy,type:n,compare:e===void 0?null:e}};Ge.startTransition=function(n){var e=Rl.transition;Rl.transition={};try{n()}finally{Rl.transition=e}};Ge.unstable_act=h_;Ge.useCallback=function(n,e){return rn.current.useCallback(n,e)};Ge.useContext=function(n){return rn.current.useContext(n)};Ge.useDebugValue=function(){};Ge.useDeferredValue=function(n){return rn.current.useDeferredValue(n)};Ge.useEffect=function(n,e){return rn.current.useEffect(n,e)};Ge.useId=function(){return rn.current.useId()};Ge.useImperativeHandle=function(n,e,t){return rn.current.useImperativeHandle(n,e,t)};Ge.useInsertionEffect=function(n,e){return rn.current.useInsertionEffect(n,e)};Ge.useLayoutEffect=function(n,e){return rn.current.useLayoutEffect(n,e)};Ge.useMemo=function(n,e){return rn.current.useMemo(n,e)};Ge.useReducer=function(n,e,t){return rn.current.useReducer(n,e,t)};Ge.useRef=function(n){return rn.current.useRef(n)};Ge.useState=function(n){return rn.current.useState(n)};Ge.useSyncExternalStore=function(n,e,t){return rn.current.useSyncExternalStore(n,e,t)};Ge.useTransition=function(){return rn.current.useTransition()};Ge.version="18.3.1";s_.exports=Ge;var Ne=s_.exports;/**
 * @license React
 * react-jsx-runtime.production.min.js
 *
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */var Ry=Ne,Cy=Symbol.for("react.element"),by=Symbol.for("react.fragment"),Py=Object.prototype.hasOwnProperty,Ly=Ry.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED.ReactCurrentOwner,Iy={key:!0,ref:!0,__self:!0,__source:!0};function p_(n,e,t){var i,r={},s=null,o=null;t!==void 0&&(s=""+t),e.key!==void 0&&(s=""+e.key),e.ref!==void 0&&(o=e.ref);for(i in e)Py.call(e,i)&&!Iy.hasOwnProperty(i)&&(r[i]=e[i]);if(n&&n.defaultProps)for(i in e=n.defaultProps,e)r[i]===void 0&&(r[i]=e[i]);return{$$typeof:Cy,type:n,key:s,ref:o,props:r,_owner:Ly.current}}Tc.Fragment=by;Tc.jsx=p_;Tc.jsxs=p_;r_.exports=Tc;var H=r_.exports,m_={exports:{}},Mn={},g_={exports:{}},__={};/**
 * @license React
 * scheduler.production.min.js
 *
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */(function(n){function e(N,K){var Q=N.length;N.push(K);e:for(;0<Q;){var oe=Q-1>>>1,Me=N[oe];if(0<r(Me,K))N[oe]=K,N[Q]=Me,Q=oe;else break e}}function t(N){return N.length===0?null:N[0]}function i(N){if(N.length===0)return null;var K=N[0],Q=N.pop();if(Q!==K){N[0]=Q;e:for(var oe=0,Me=N.length,je=Me>>>1;oe<je;){var G=2*(oe+1)-1,ne=N[G],he=G+1,de=N[he];if(0>r(ne,Q))he<Me&&0>r(de,ne)?(N[oe]=de,N[he]=Q,oe=he):(N[oe]=ne,N[G]=Q,oe=G);else if(he<Me&&0>r(de,Q))N[oe]=de,N[he]=Q,oe=he;else break e}}return K}function r(N,K){var Q=N.sortIndex-K.sortIndex;return Q!==0?Q:N.id-K.id}if(typeof performance=="object"&&typeof performance.now=="function"){var s=performance;n.unstable_now=function(){return s.now()}}else{var o=Date,a=o.now();n.unstable_now=function(){return o.now()-a}}var l=[],c=[],u=1,d=null,f=3,p=!1,g=!1,y=!1,m=typeof setTimeout=="function"?setTimeout:null,h=typeof clearTimeout=="function"?clearTimeout:null,_=typeof setImmediate<"u"?setImmediate:null;typeof navigator<"u"&&navigator.scheduling!==void 0&&navigator.scheduling.isInputPending!==void 0&&navigator.scheduling.isInputPending.bind(navigator.scheduling);function v(N){for(var K=t(c);K!==null;){if(K.callback===null)i(c);else if(K.startTime<=N)i(c),K.sortIndex=K.expirationTime,e(l,K);else break;K=t(c)}}function x(N){if(y=!1,v(N),!g)if(t(l)!==null)g=!0,W(R);else{var K=t(c);K!==null&&$(x,K.startTime-N)}}function R(N,K){g=!1,y&&(y=!1,h(b),b=-1),p=!0;var Q=f;try{for(v(K),d=t(l);d!==null&&(!(d.expirationTime>K)||N&&!I());){var oe=d.callback;if(typeof oe=="function"){d.callback=null,f=d.priorityLevel;var Me=oe(d.expirationTime<=K);K=n.unstable_now(),typeof Me=="function"?d.callback=Me:d===t(l)&&i(l),v(K)}else i(l);d=t(l)}if(d!==null)var je=!0;else{var G=t(c);G!==null&&$(x,G.startTime-K),je=!1}return je}finally{d=null,f=Q,p=!1}}var A=!1,w=null,b=-1,T=5,M=-1;function I(){return!(n.unstable_now()-M<T)}function V(){if(w!==null){var N=n.unstable_now();M=N;var K=!0;try{K=w(!0,N)}finally{K?B():(A=!1,w=null)}}else A=!1}var B;if(typeof _=="function")B=function(){_(V)};else if(typeof MessageChannel<"u"){var j=new MessageChannel,Y=j.port2;j.port1.onmessage=V,B=function(){Y.postMessage(null)}}else B=function(){m(V,0)};function W(N){w=N,A||(A=!0,B())}function $(N,K){b=m(function(){N(n.unstable_now())},K)}n.unstable_IdlePriority=5,n.unstable_ImmediatePriority=1,n.unstable_LowPriority=4,n.unstable_NormalPriority=3,n.unstable_Profiling=null,n.unstable_UserBlockingPriority=2,n.unstable_cancelCallback=function(N){N.callback=null},n.unstable_continueExecution=function(){g||p||(g=!0,W(R))},n.unstable_forceFrameRate=function(N){0>N||125<N?console.error("forceFrameRate takes a positive int between 0 and 125, forcing frame rates higher than 125 fps is not supported"):T=0<N?Math.floor(1e3/N):5},n.unstable_getCurrentPriorityLevel=function(){return f},n.unstable_getFirstCallbackNode=function(){return t(l)},n.unstable_next=function(N){switch(f){case 1:case 2:case 3:var K=3;break;default:K=f}var Q=f;f=K;try{return N()}finally{f=Q}},n.unstable_pauseExecution=function(){},n.unstable_requestPaint=function(){},n.unstable_runWithPriority=function(N,K){switch(N){case 1:case 2:case 3:case 4:case 5:break;default:N=3}var Q=f;f=N;try{return K()}finally{f=Q}},n.unstable_scheduleCallback=function(N,K,Q){var oe=n.unstable_now();switch(typeof Q=="object"&&Q!==null?(Q=Q.delay,Q=typeof Q=="number"&&0<Q?oe+Q:oe):Q=oe,N){case 1:var Me=-1;break;case 2:Me=250;break;case 5:Me=1073741823;break;case 4:Me=1e4;break;default:Me=5e3}return Me=Q+Me,N={id:u++,callback:K,priorityLevel:N,startTime:Q,expirationTime:Me,sortIndex:-1},Q>oe?(N.sortIndex=Q,e(c,N),t(l)===null&&N===t(c)&&(y?(h(b),b=-1):y=!0,$(x,Q-oe))):(N.sortIndex=Me,e(l,N),g||p||(g=!0,W(R))),N},n.unstable_shouldYield=I,n.unstable_wrapCallback=function(N){var K=f;return function(){var Q=f;f=K;try{return N.apply(this,arguments)}finally{f=Q}}}})(__);g_.exports=__;var Ny=g_.exports;/**
 * @license React
 * react-dom.production.min.js
 *
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */var Dy=Ne,Sn=Ny;function te(n){for(var e="https://reactjs.org/docs/error-decoder.html?invariant="+n,t=1;t<arguments.length;t++)e+="&args[]="+encodeURIComponent(arguments[t]);return"Minified React error #"+n+"; visit "+e+" for the full message or use the non-minified dev environment for full errors and additional helpful warnings."}var v_=new Set,$o={};function Gr(n,e){Bs(n,e),Bs(n+"Capture",e)}function Bs(n,e){for($o[n]=e,n=0;n<e.length;n++)v_.add(e[n])}var Ci=!(typeof window>"u"||typeof window.document>"u"||typeof window.document.createElement>"u"),ud=Object.prototype.hasOwnProperty,Uy=/^[:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD][:A-Z_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u02FF\u0370-\u037D\u037F-\u1FFF\u200C-\u200D\u2070-\u218F\u2C00-\u2FEF\u3001-\uD7FF\uF900-\uFDCF\uFDF0-\uFFFD\-.0-9\u00B7\u0300-\u036F\u203F-\u2040]*$/,gp={},_p={};function Fy(n){return ud.call(_p,n)?!0:ud.call(gp,n)?!1:Uy.test(n)?_p[n]=!0:(gp[n]=!0,!1)}function Oy(n,e,t,i){if(t!==null&&t.type===0)return!1;switch(typeof e){case"function":case"symbol":return!0;case"boolean":return i?!1:t!==null?!t.acceptsBooleans:(n=n.toLowerCase().slice(0,5),n!=="data-"&&n!=="aria-");default:return!1}}function ky(n,e,t,i){if(e===null||typeof e>"u"||Oy(n,e,t,i))return!0;if(i)return!1;if(t!==null)switch(t.type){case 3:return!e;case 4:return e===!1;case 5:return isNaN(e);case 6:return isNaN(e)||1>e}return!1}function sn(n,e,t,i,r,s,o){this.acceptsBooleans=e===2||e===3||e===4,this.attributeName=i,this.attributeNamespace=r,this.mustUseProperty=t,this.propertyName=n,this.type=e,this.sanitizeURL=s,this.removeEmptyString=o}var Ht={};"children dangerouslySetInnerHTML defaultValue defaultChecked innerHTML suppressContentEditableWarning suppressHydrationWarning style".split(" ").forEach(function(n){Ht[n]=new sn(n,0,!1,n,null,!1,!1)});[["acceptCharset","accept-charset"],["className","class"],["htmlFor","for"],["httpEquiv","http-equiv"]].forEach(function(n){var e=n[0];Ht[e]=new sn(e,1,!1,n[1],null,!1,!1)});["contentEditable","draggable","spellCheck","value"].forEach(function(n){Ht[n]=new sn(n,2,!1,n.toLowerCase(),null,!1,!1)});["autoReverse","externalResourcesRequired","focusable","preserveAlpha"].forEach(function(n){Ht[n]=new sn(n,2,!1,n,null,!1,!1)});"allowFullScreen async autoFocus autoPlay controls default defer disabled disablePictureInPicture disableRemotePlayback formNoValidate hidden loop noModule noValidate open playsInline readOnly required reversed scoped seamless itemScope".split(" ").forEach(function(n){Ht[n]=new sn(n,3,!1,n.toLowerCase(),null,!1,!1)});["checked","multiple","muted","selected"].forEach(function(n){Ht[n]=new sn(n,3,!0,n,null,!1,!1)});["capture","download"].forEach(function(n){Ht[n]=new sn(n,4,!1,n,null,!1,!1)});["cols","rows","size","span"].forEach(function(n){Ht[n]=new sn(n,6,!1,n,null,!1,!1)});["rowSpan","start"].forEach(function(n){Ht[n]=new sn(n,5,!1,n.toLowerCase(),null,!1,!1)});var jf=/[\-:]([a-z])/g;function Yf(n){return n[1].toUpperCase()}"accent-height alignment-baseline arabic-form baseline-shift cap-height clip-path clip-rule color-interpolation color-interpolation-filters color-profile color-rendering dominant-baseline enable-background fill-opacity fill-rule flood-color flood-opacity font-family font-size font-size-adjust font-stretch font-style font-variant font-weight glyph-name glyph-orientation-horizontal glyph-orientation-vertical horiz-adv-x horiz-origin-x image-rendering letter-spacing lighting-color marker-end marker-mid marker-start overline-position overline-thickness paint-order panose-1 pointer-events rendering-intent shape-rendering stop-color stop-opacity strikethrough-position strikethrough-thickness stroke-dasharray stroke-dashoffset stroke-linecap stroke-linejoin stroke-miterlimit stroke-opacity stroke-width text-anchor text-decoration text-rendering underline-position underline-thickness unicode-bidi unicode-range units-per-em v-alphabetic v-hanging v-ideographic v-mathematical vector-effect vert-adv-y vert-origin-x vert-origin-y word-spacing writing-mode xmlns:xlink x-height".split(" ").forEach(function(n){var e=n.replace(jf,Yf);Ht[e]=new sn(e,1,!1,n,null,!1,!1)});"xlink:actuate xlink:arcrole xlink:role xlink:show xlink:title xlink:type".split(" ").forEach(function(n){var e=n.replace(jf,Yf);Ht[e]=new sn(e,1,!1,n,"http://www.w3.org/1999/xlink",!1,!1)});["xml:base","xml:lang","xml:space"].forEach(function(n){var e=n.replace(jf,Yf);Ht[e]=new sn(e,1,!1,n,"http://www.w3.org/XML/1998/namespace",!1,!1)});["tabIndex","crossOrigin"].forEach(function(n){Ht[n]=new sn(n,1,!1,n.toLowerCase(),null,!1,!1)});Ht.xlinkHref=new sn("xlinkHref",1,!1,"xlink:href","http://www.w3.org/1999/xlink",!0,!1);["src","href","action","formAction"].forEach(function(n){Ht[n]=new sn(n,1,!1,n.toLowerCase(),null,!0,!0)});function Kf(n,e,t,i){var r=Ht.hasOwnProperty(e)?Ht[e]:null;(r!==null?r.type!==0:i||!(2<e.length)||e[0]!=="o"&&e[0]!=="O"||e[1]!=="n"&&e[1]!=="N")&&(ky(e,t,r,i)&&(t=null),i||r===null?Fy(e)&&(t===null?n.removeAttribute(e):n.setAttribute(e,""+t)):r.mustUseProperty?n[r.propertyName]=t===null?r.type===3?!1:"":t:(e=r.attributeName,i=r.attributeNamespace,t===null?n.removeAttribute(e):(r=r.type,t=r===3||r===4&&t===!0?"":""+t,i?n.setAttributeNS(i,e,t):n.setAttribute(e,t))))}var Ui=Dy.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED,La=Symbol.for("react.element"),ms=Symbol.for("react.portal"),gs=Symbol.for("react.fragment"),qf=Symbol.for("react.strict_mode"),dd=Symbol.for("react.profiler"),y_=Symbol.for("react.provider"),x_=Symbol.for("react.context"),$f=Symbol.for("react.forward_ref"),fd=Symbol.for("react.suspense"),hd=Symbol.for("react.suspense_list"),Zf=Symbol.for("react.memo"),ji=Symbol.for("react.lazy"),S_=Symbol.for("react.offscreen"),vp=Symbol.iterator;function uo(n){return n===null||typeof n!="object"?null:(n=vp&&n[vp]||n["@@iterator"],typeof n=="function"?n:null)}var mt=Object.assign,$c;function Io(n){if($c===void 0)try{throw Error()}catch(t){var e=t.stack.trim().match(/\n( *(at )?)/);$c=e&&e[1]||""}return`
`+$c+n}var Zc=!1;function Qc(n,e){if(!n||Zc)return"";Zc=!0;var t=Error.prepareStackTrace;Error.prepareStackTrace=void 0;try{if(e)if(e=function(){throw Error()},Object.defineProperty(e.prototype,"props",{set:function(){throw Error()}}),typeof Reflect=="object"&&Reflect.construct){try{Reflect.construct(e,[])}catch(c){var i=c}Reflect.construct(n,[],e)}else{try{e.call()}catch(c){i=c}n.call(e.prototype)}else{try{throw Error()}catch(c){i=c}n()}}catch(c){if(c&&i&&typeof c.stack=="string"){for(var r=c.stack.split(`
`),s=i.stack.split(`
`),o=r.length-1,a=s.length-1;1<=o&&0<=a&&r[o]!==s[a];)a--;for(;1<=o&&0<=a;o--,a--)if(r[o]!==s[a]){if(o!==1||a!==1)do if(o--,a--,0>a||r[o]!==s[a]){var l=`
`+r[o].replace(" at new "," at ");return n.displayName&&l.includes("<anonymous>")&&(l=l.replace("<anonymous>",n.displayName)),l}while(1<=o&&0<=a);break}}}finally{Zc=!1,Error.prepareStackTrace=t}return(n=n?n.displayName||n.name:"")?Io(n):""}function By(n){switch(n.tag){case 5:return Io(n.type);case 16:return Io("Lazy");case 13:return Io("Suspense");case 19:return Io("SuspenseList");case 0:case 2:case 15:return n=Qc(n.type,!1),n;case 11:return n=Qc(n.type.render,!1),n;case 1:return n=Qc(n.type,!0),n;default:return""}}function pd(n){if(n==null)return null;if(typeof n=="function")return n.displayName||n.name||null;if(typeof n=="string")return n;switch(n){case gs:return"Fragment";case ms:return"Portal";case dd:return"Profiler";case qf:return"StrictMode";case fd:return"Suspense";case hd:return"SuspenseList"}if(typeof n=="object")switch(n.$$typeof){case x_:return(n.displayName||"Context")+".Consumer";case y_:return(n._context.displayName||"Context")+".Provider";case $f:var e=n.render;return n=n.displayName,n||(n=e.displayName||e.name||"",n=n!==""?"ForwardRef("+n+")":"ForwardRef"),n;case Zf:return e=n.displayName||null,e!==null?e:pd(n.type)||"Memo";case ji:e=n._payload,n=n._init;try{return pd(n(e))}catch{}}return null}function zy(n){var e=n.type;switch(n.tag){case 24:return"Cache";case 9:return(e.displayName||"Context")+".Consumer";case 10:return(e._context.displayName||"Context")+".Provider";case 18:return"DehydratedFragment";case 11:return n=e.render,n=n.displayName||n.name||"",e.displayName||(n!==""?"ForwardRef("+n+")":"ForwardRef");case 7:return"Fragment";case 5:return e;case 4:return"Portal";case 3:return"Root";case 6:return"Text";case 16:return pd(e);case 8:return e===qf?"StrictMode":"Mode";case 22:return"Offscreen";case 12:return"Profiler";case 21:return"Scope";case 13:return"Suspense";case 19:return"SuspenseList";case 25:return"TracingMarker";case 1:case 0:case 17:case 2:case 14:case 15:if(typeof e=="function")return e.displayName||e.name||null;if(typeof e=="string")return e}return null}function dr(n){switch(typeof n){case"boolean":case"number":case"string":case"undefined":return n;case"object":return n;default:return""}}function M_(n){var e=n.type;return(n=n.nodeName)&&n.toLowerCase()==="input"&&(e==="checkbox"||e==="radio")}function Hy(n){var e=M_(n)?"checked":"value",t=Object.getOwnPropertyDescriptor(n.constructor.prototype,e),i=""+n[e];if(!n.hasOwnProperty(e)&&typeof t<"u"&&typeof t.get=="function"&&typeof t.set=="function"){var r=t.get,s=t.set;return Object.defineProperty(n,e,{configurable:!0,get:function(){return r.call(this)},set:function(o){i=""+o,s.call(this,o)}}),Object.defineProperty(n,e,{enumerable:t.enumerable}),{getValue:function(){return i},setValue:function(o){i=""+o},stopTracking:function(){n._valueTracker=null,delete n[e]}}}}function Ia(n){n._valueTracker||(n._valueTracker=Hy(n))}function E_(n){if(!n)return!1;var e=n._valueTracker;if(!e)return!0;var t=e.getValue(),i="";return n&&(i=M_(n)?n.checked?"true":"false":n.value),n=i,n!==t?(e.setValue(n),!0):!1}function Xl(n){if(n=n||(typeof document<"u"?document:void 0),typeof n>"u")return null;try{return n.activeElement||n.body}catch{return n.body}}function md(n,e){var t=e.checked;return mt({},e,{defaultChecked:void 0,defaultValue:void 0,value:void 0,checked:t??n._wrapperState.initialChecked})}function yp(n,e){var t=e.defaultValue==null?"":e.defaultValue,i=e.checked!=null?e.checked:e.defaultChecked;t=dr(e.value!=null?e.value:t),n._wrapperState={initialChecked:i,initialValue:t,controlled:e.type==="checkbox"||e.type==="radio"?e.checked!=null:e.value!=null}}function T_(n,e){e=e.checked,e!=null&&Kf(n,"checked",e,!1)}function gd(n,e){T_(n,e);var t=dr(e.value),i=e.type;if(t!=null)i==="number"?(t===0&&n.value===""||n.value!=t)&&(n.value=""+t):n.value!==""+t&&(n.value=""+t);else if(i==="submit"||i==="reset"){n.removeAttribute("value");return}e.hasOwnProperty("value")?_d(n,e.type,t):e.hasOwnProperty("defaultValue")&&_d(n,e.type,dr(e.defaultValue)),e.checked==null&&e.defaultChecked!=null&&(n.defaultChecked=!!e.defaultChecked)}function xp(n,e,t){if(e.hasOwnProperty("value")||e.hasOwnProperty("defaultValue")){var i=e.type;if(!(i!=="submit"&&i!=="reset"||e.value!==void 0&&e.value!==null))return;e=""+n._wrapperState.initialValue,t||e===n.value||(n.value=e),n.defaultValue=e}t=n.name,t!==""&&(n.name=""),n.defaultChecked=!!n._wrapperState.initialChecked,t!==""&&(n.name=t)}function _d(n,e,t){(e!=="number"||Xl(n.ownerDocument)!==n)&&(t==null?n.defaultValue=""+n._wrapperState.initialValue:n.defaultValue!==""+t&&(n.defaultValue=""+t))}var No=Array.isArray;function Cs(n,e,t,i){if(n=n.options,e){e={};for(var r=0;r<t.length;r++)e["$"+t[r]]=!0;for(t=0;t<n.length;t++)r=e.hasOwnProperty("$"+n[t].value),n[t].selected!==r&&(n[t].selected=r),r&&i&&(n[t].defaultSelected=!0)}else{for(t=""+dr(t),e=null,r=0;r<n.length;r++){if(n[r].value===t){n[r].selected=!0,i&&(n[r].defaultSelected=!0);return}e!==null||n[r].disabled||(e=n[r])}e!==null&&(e.selected=!0)}}function vd(n,e){if(e.dangerouslySetInnerHTML!=null)throw Error(te(91));return mt({},e,{value:void 0,defaultValue:void 0,children:""+n._wrapperState.initialValue})}function Sp(n,e){var t=e.value;if(t==null){if(t=e.children,e=e.defaultValue,t!=null){if(e!=null)throw Error(te(92));if(No(t)){if(1<t.length)throw Error(te(93));t=t[0]}e=t}e==null&&(e=""),t=e}n._wrapperState={initialValue:dr(t)}}function w_(n,e){var t=dr(e.value),i=dr(e.defaultValue);t!=null&&(t=""+t,t!==n.value&&(n.value=t),e.defaultValue==null&&n.defaultValue!==t&&(n.defaultValue=t)),i!=null&&(n.defaultValue=""+i)}function Mp(n){var e=n.textContent;e===n._wrapperState.initialValue&&e!==""&&e!==null&&(n.value=e)}function A_(n){switch(n){case"svg":return"http://www.w3.org/2000/svg";case"math":return"http://www.w3.org/1998/Math/MathML";default:return"http://www.w3.org/1999/xhtml"}}function yd(n,e){return n==null||n==="http://www.w3.org/1999/xhtml"?A_(e):n==="http://www.w3.org/2000/svg"&&e==="foreignObject"?"http://www.w3.org/1999/xhtml":n}var Na,R_=function(n){return typeof MSApp<"u"&&MSApp.execUnsafeLocalFunction?function(e,t,i,r){MSApp.execUnsafeLocalFunction(function(){return n(e,t,i,r)})}:n}(function(n,e){if(n.namespaceURI!=="http://www.w3.org/2000/svg"||"innerHTML"in n)n.innerHTML=e;else{for(Na=Na||document.createElement("div"),Na.innerHTML="<svg>"+e.valueOf().toString()+"</svg>",e=Na.firstChild;n.firstChild;)n.removeChild(n.firstChild);for(;e.firstChild;)n.appendChild(e.firstChild)}});function Zo(n,e){if(e){var t=n.firstChild;if(t&&t===n.lastChild&&t.nodeType===3){t.nodeValue=e;return}}n.textContent=e}var ko={animationIterationCount:!0,aspectRatio:!0,borderImageOutset:!0,borderImageSlice:!0,borderImageWidth:!0,boxFlex:!0,boxFlexGroup:!0,boxOrdinalGroup:!0,columnCount:!0,columns:!0,flex:!0,flexGrow:!0,flexPositive:!0,flexShrink:!0,flexNegative:!0,flexOrder:!0,gridArea:!0,gridRow:!0,gridRowEnd:!0,gridRowSpan:!0,gridRowStart:!0,gridColumn:!0,gridColumnEnd:!0,gridColumnSpan:!0,gridColumnStart:!0,fontWeight:!0,lineClamp:!0,lineHeight:!0,opacity:!0,order:!0,orphans:!0,tabSize:!0,widows:!0,zIndex:!0,zoom:!0,fillOpacity:!0,floodOpacity:!0,stopOpacity:!0,strokeDasharray:!0,strokeDashoffset:!0,strokeMiterlimit:!0,strokeOpacity:!0,strokeWidth:!0},Vy=["Webkit","ms","Moz","O"];Object.keys(ko).forEach(function(n){Vy.forEach(function(e){e=e+n.charAt(0).toUpperCase()+n.substring(1),ko[e]=ko[n]})});function C_(n,e,t){return e==null||typeof e=="boolean"||e===""?"":t||typeof e!="number"||e===0||ko.hasOwnProperty(n)&&ko[n]?(""+e).trim():e+"px"}function b_(n,e){n=n.style;for(var t in e)if(e.hasOwnProperty(t)){var i=t.indexOf("--")===0,r=C_(t,e[t],i);t==="float"&&(t="cssFloat"),i?n.setProperty(t,r):n[t]=r}}var Gy=mt({menuitem:!0},{area:!0,base:!0,br:!0,col:!0,embed:!0,hr:!0,img:!0,input:!0,keygen:!0,link:!0,meta:!0,param:!0,source:!0,track:!0,wbr:!0});function xd(n,e){if(e){if(Gy[n]&&(e.children!=null||e.dangerouslySetInnerHTML!=null))throw Error(te(137,n));if(e.dangerouslySetInnerHTML!=null){if(e.children!=null)throw Error(te(60));if(typeof e.dangerouslySetInnerHTML!="object"||!("__html"in e.dangerouslySetInnerHTML))throw Error(te(61))}if(e.style!=null&&typeof e.style!="object")throw Error(te(62))}}function Sd(n,e){if(n.indexOf("-")===-1)return typeof e.is=="string";switch(n){case"annotation-xml":case"color-profile":case"font-face":case"font-face-src":case"font-face-uri":case"font-face-format":case"font-face-name":case"missing-glyph":return!1;default:return!0}}var Md=null;function Qf(n){return n=n.target||n.srcElement||window,n.correspondingUseElement&&(n=n.correspondingUseElement),n.nodeType===3?n.parentNode:n}var Ed=null,bs=null,Ps=null;function Ep(n){if(n=Sa(n)){if(typeof Ed!="function")throw Error(te(280));var e=n.stateNode;e&&(e=bc(e),Ed(n.stateNode,n.type,e))}}function P_(n){bs?Ps?Ps.push(n):Ps=[n]:bs=n}function L_(){if(bs){var n=bs,e=Ps;if(Ps=bs=null,Ep(n),e)for(n=0;n<e.length;n++)Ep(e[n])}}function I_(n,e){return n(e)}function N_(){}var Jc=!1;function D_(n,e,t){if(Jc)return n(e,t);Jc=!0;try{return I_(n,e,t)}finally{Jc=!1,(bs!==null||Ps!==null)&&(N_(),L_())}}function Qo(n,e){var t=n.stateNode;if(t===null)return null;var i=bc(t);if(i===null)return null;t=i[e];e:switch(e){case"onClick":case"onClickCapture":case"onDoubleClick":case"onDoubleClickCapture":case"onMouseDown":case"onMouseDownCapture":case"onMouseMove":case"onMouseMoveCapture":case"onMouseUp":case"onMouseUpCapture":case"onMouseEnter":(i=!i.disabled)||(n=n.type,i=!(n==="button"||n==="input"||n==="select"||n==="textarea")),n=!i;break e;default:n=!1}if(n)return null;if(t&&typeof t!="function")throw Error(te(231,e,typeof t));return t}var Td=!1;if(Ci)try{var fo={};Object.defineProperty(fo,"passive",{get:function(){Td=!0}}),window.addEventListener("test",fo,fo),window.removeEventListener("test",fo,fo)}catch{Td=!1}function Wy(n,e,t,i,r,s,o,a,l){var c=Array.prototype.slice.call(arguments,3);try{e.apply(t,c)}catch(u){this.onError(u)}}var Bo=!1,jl=null,Yl=!1,wd=null,Xy={onError:function(n){Bo=!0,jl=n}};function jy(n,e,t,i,r,s,o,a,l){Bo=!1,jl=null,Wy.apply(Xy,arguments)}function Yy(n,e,t,i,r,s,o,a,l){if(jy.apply(this,arguments),Bo){if(Bo){var c=jl;Bo=!1,jl=null}else throw Error(te(198));Yl||(Yl=!0,wd=c)}}function Wr(n){var e=n,t=n;if(n.alternate)for(;e.return;)e=e.return;else{n=e;do e=n,e.flags&4098&&(t=e.return),n=e.return;while(n)}return e.tag===3?t:null}function U_(n){if(n.tag===13){var e=n.memoizedState;if(e===null&&(n=n.alternate,n!==null&&(e=n.memoizedState)),e!==null)return e.dehydrated}return null}function Tp(n){if(Wr(n)!==n)throw Error(te(188))}function Ky(n){var e=n.alternate;if(!e){if(e=Wr(n),e===null)throw Error(te(188));return e!==n?null:n}for(var t=n,i=e;;){var r=t.return;if(r===null)break;var s=r.alternate;if(s===null){if(i=r.return,i!==null){t=i;continue}break}if(r.child===s.child){for(s=r.child;s;){if(s===t)return Tp(r),n;if(s===i)return Tp(r),e;s=s.sibling}throw Error(te(188))}if(t.return!==i.return)t=r,i=s;else{for(var o=!1,a=r.child;a;){if(a===t){o=!0,t=r,i=s;break}if(a===i){o=!0,i=r,t=s;break}a=a.sibling}if(!o){for(a=s.child;a;){if(a===t){o=!0,t=s,i=r;break}if(a===i){o=!0,i=s,t=r;break}a=a.sibling}if(!o)throw Error(te(189))}}if(t.alternate!==i)throw Error(te(190))}if(t.tag!==3)throw Error(te(188));return t.stateNode.current===t?n:e}function F_(n){return n=Ky(n),n!==null?O_(n):null}function O_(n){if(n.tag===5||n.tag===6)return n;for(n=n.child;n!==null;){var e=O_(n);if(e!==null)return e;n=n.sibling}return null}var k_=Sn.unstable_scheduleCallback,wp=Sn.unstable_cancelCallback,qy=Sn.unstable_shouldYield,$y=Sn.unstable_requestPaint,xt=Sn.unstable_now,Zy=Sn.unstable_getCurrentPriorityLevel,Jf=Sn.unstable_ImmediatePriority,B_=Sn.unstable_UserBlockingPriority,Kl=Sn.unstable_NormalPriority,Qy=Sn.unstable_LowPriority,z_=Sn.unstable_IdlePriority,wc=null,si=null;function Jy(n){if(si&&typeof si.onCommitFiberRoot=="function")try{si.onCommitFiberRoot(wc,n,void 0,(n.current.flags&128)===128)}catch{}}var jn=Math.clz32?Math.clz32:nx,ex=Math.log,tx=Math.LN2;function nx(n){return n>>>=0,n===0?32:31-(ex(n)/tx|0)|0}var Da=64,Ua=4194304;function Do(n){switch(n&-n){case 1:return 1;case 2:return 2;case 4:return 4;case 8:return 8;case 16:return 16;case 32:return 32;case 64:case 128:case 256:case 512:case 1024:case 2048:case 4096:case 8192:case 16384:case 32768:case 65536:case 131072:case 262144:case 524288:case 1048576:case 2097152:return n&4194240;case 4194304:case 8388608:case 16777216:case 33554432:case 67108864:return n&130023424;case 134217728:return 134217728;case 268435456:return 268435456;case 536870912:return 536870912;case 1073741824:return 1073741824;default:return n}}function ql(n,e){var t=n.pendingLanes;if(t===0)return 0;var i=0,r=n.suspendedLanes,s=n.pingedLanes,o=t&268435455;if(o!==0){var a=o&~r;a!==0?i=Do(a):(s&=o,s!==0&&(i=Do(s)))}else o=t&~r,o!==0?i=Do(o):s!==0&&(i=Do(s));if(i===0)return 0;if(e!==0&&e!==i&&!(e&r)&&(r=i&-i,s=e&-e,r>=s||r===16&&(s&4194240)!==0))return e;if(i&4&&(i|=t&16),e=n.entangledLanes,e!==0)for(n=n.entanglements,e&=i;0<e;)t=31-jn(e),r=1<<t,i|=n[t],e&=~r;return i}function ix(n,e){switch(n){case 1:case 2:case 4:return e+250;case 8:case 16:case 32:case 64:case 128:case 256:case 512:case 1024:case 2048:case 4096:case 8192:case 16384:case 32768:case 65536:case 131072:case 262144:case 524288:case 1048576:case 2097152:return e+5e3;case 4194304:case 8388608:case 16777216:case 33554432:case 67108864:return-1;case 134217728:case 268435456:case 536870912:case 1073741824:return-1;default:return-1}}function rx(n,e){for(var t=n.suspendedLanes,i=n.pingedLanes,r=n.expirationTimes,s=n.pendingLanes;0<s;){var o=31-jn(s),a=1<<o,l=r[o];l===-1?(!(a&t)||a&i)&&(r[o]=ix(a,e)):l<=e&&(n.expiredLanes|=a),s&=~a}}function Ad(n){return n=n.pendingLanes&-1073741825,n!==0?n:n&1073741824?1073741824:0}function H_(){var n=Da;return Da<<=1,!(Da&4194240)&&(Da=64),n}function eu(n){for(var e=[],t=0;31>t;t++)e.push(n);return e}function ya(n,e,t){n.pendingLanes|=e,e!==536870912&&(n.suspendedLanes=0,n.pingedLanes=0),n=n.eventTimes,e=31-jn(e),n[e]=t}function sx(n,e){var t=n.pendingLanes&~e;n.pendingLanes=e,n.suspendedLanes=0,n.pingedLanes=0,n.expiredLanes&=e,n.mutableReadLanes&=e,n.entangledLanes&=e,e=n.entanglements;var i=n.eventTimes;for(n=n.expirationTimes;0<t;){var r=31-jn(t),s=1<<r;e[r]=0,i[r]=-1,n[r]=-1,t&=~s}}function eh(n,e){var t=n.entangledLanes|=e;for(n=n.entanglements;t;){var i=31-jn(t),r=1<<i;r&e|n[i]&e&&(n[i]|=e),t&=~r}}var it=0;function V_(n){return n&=-n,1<n?4<n?n&268435455?16:536870912:4:1}var G_,th,W_,X_,j_,Rd=!1,Fa=[],tr=null,nr=null,ir=null,Jo=new Map,ea=new Map,Ki=[],ox="mousedown mouseup touchcancel touchend touchstart auxclick dblclick pointercancel pointerdown pointerup dragend dragstart drop compositionend compositionstart keydown keypress keyup input textInput copy cut paste click change contextmenu reset submit".split(" ");function Ap(n,e){switch(n){case"focusin":case"focusout":tr=null;break;case"dragenter":case"dragleave":nr=null;break;case"mouseover":case"mouseout":ir=null;break;case"pointerover":case"pointerout":Jo.delete(e.pointerId);break;case"gotpointercapture":case"lostpointercapture":ea.delete(e.pointerId)}}function ho(n,e,t,i,r,s){return n===null||n.nativeEvent!==s?(n={blockedOn:e,domEventName:t,eventSystemFlags:i,nativeEvent:s,targetContainers:[r]},e!==null&&(e=Sa(e),e!==null&&th(e)),n):(n.eventSystemFlags|=i,e=n.targetContainers,r!==null&&e.indexOf(r)===-1&&e.push(r),n)}function ax(n,e,t,i,r){switch(e){case"focusin":return tr=ho(tr,n,e,t,i,r),!0;case"dragenter":return nr=ho(nr,n,e,t,i,r),!0;case"mouseover":return ir=ho(ir,n,e,t,i,r),!0;case"pointerover":var s=r.pointerId;return Jo.set(s,ho(Jo.get(s)||null,n,e,t,i,r)),!0;case"gotpointercapture":return s=r.pointerId,ea.set(s,ho(ea.get(s)||null,n,e,t,i,r)),!0}return!1}function Y_(n){var e=Lr(n.target);if(e!==null){var t=Wr(e);if(t!==null){if(e=t.tag,e===13){if(e=U_(t),e!==null){n.blockedOn=e,j_(n.priority,function(){W_(t)});return}}else if(e===3&&t.stateNode.current.memoizedState.isDehydrated){n.blockedOn=t.tag===3?t.stateNode.containerInfo:null;return}}}n.blockedOn=null}function Cl(n){if(n.blockedOn!==null)return!1;for(var e=n.targetContainers;0<e.length;){var t=Cd(n.domEventName,n.eventSystemFlags,e[0],n.nativeEvent);if(t===null){t=n.nativeEvent;var i=new t.constructor(t.type,t);Md=i,t.target.dispatchEvent(i),Md=null}else return e=Sa(t),e!==null&&th(e),n.blockedOn=t,!1;e.shift()}return!0}function Rp(n,e,t){Cl(n)&&t.delete(e)}function lx(){Rd=!1,tr!==null&&Cl(tr)&&(tr=null),nr!==null&&Cl(nr)&&(nr=null),ir!==null&&Cl(ir)&&(ir=null),Jo.forEach(Rp),ea.forEach(Rp)}function po(n,e){n.blockedOn===e&&(n.blockedOn=null,Rd||(Rd=!0,Sn.unstable_scheduleCallback(Sn.unstable_NormalPriority,lx)))}function ta(n){function e(r){return po(r,n)}if(0<Fa.length){po(Fa[0],n);for(var t=1;t<Fa.length;t++){var i=Fa[t];i.blockedOn===n&&(i.blockedOn=null)}}for(tr!==null&&po(tr,n),nr!==null&&po(nr,n),ir!==null&&po(ir,n),Jo.forEach(e),ea.forEach(e),t=0;t<Ki.length;t++)i=Ki[t],i.blockedOn===n&&(i.blockedOn=null);for(;0<Ki.length&&(t=Ki[0],t.blockedOn===null);)Y_(t),t.blockedOn===null&&Ki.shift()}var Ls=Ui.ReactCurrentBatchConfig,$l=!0;function cx(n,e,t,i){var r=it,s=Ls.transition;Ls.transition=null;try{it=1,nh(n,e,t,i)}finally{it=r,Ls.transition=s}}function ux(n,e,t,i){var r=it,s=Ls.transition;Ls.transition=null;try{it=4,nh(n,e,t,i)}finally{it=r,Ls.transition=s}}function nh(n,e,t,i){if($l){var r=Cd(n,e,t,i);if(r===null)uu(n,e,i,Zl,t),Ap(n,i);else if(ax(r,n,e,t,i))i.stopPropagation();else if(Ap(n,i),e&4&&-1<ox.indexOf(n)){for(;r!==null;){var s=Sa(r);if(s!==null&&G_(s),s=Cd(n,e,t,i),s===null&&uu(n,e,i,Zl,t),s===r)break;r=s}r!==null&&i.stopPropagation()}else uu(n,e,i,null,t)}}var Zl=null;function Cd(n,e,t,i){if(Zl=null,n=Qf(i),n=Lr(n),n!==null)if(e=Wr(n),e===null)n=null;else if(t=e.tag,t===13){if(n=U_(e),n!==null)return n;n=null}else if(t===3){if(e.stateNode.current.memoizedState.isDehydrated)return e.tag===3?e.stateNode.containerInfo:null;n=null}else e!==n&&(n=null);return Zl=n,null}function K_(n){switch(n){case"cancel":case"click":case"close":case"contextmenu":case"copy":case"cut":case"auxclick":case"dblclick":case"dragend":case"dragstart":case"drop":case"focusin":case"focusout":case"input":case"invalid":case"keydown":case"keypress":case"keyup":case"mousedown":case"mouseup":case"paste":case"pause":case"play":case"pointercancel":case"pointerdown":case"pointerup":case"ratechange":case"reset":case"resize":case"seeked":case"submit":case"touchcancel":case"touchend":case"touchstart":case"volumechange":case"change":case"selectionchange":case"textInput":case"compositionstart":case"compositionend":case"compositionupdate":case"beforeblur":case"afterblur":case"beforeinput":case"blur":case"fullscreenchange":case"focus":case"hashchange":case"popstate":case"select":case"selectstart":return 1;case"drag":case"dragenter":case"dragexit":case"dragleave":case"dragover":case"mousemove":case"mouseout":case"mouseover":case"pointermove":case"pointerout":case"pointerover":case"scroll":case"toggle":case"touchmove":case"wheel":case"mouseenter":case"mouseleave":case"pointerenter":case"pointerleave":return 4;case"message":switch(Zy()){case Jf:return 1;case B_:return 4;case Kl:case Qy:return 16;case z_:return 536870912;default:return 16}default:return 16}}var Zi=null,ih=null,bl=null;function q_(){if(bl)return bl;var n,e=ih,t=e.length,i,r="value"in Zi?Zi.value:Zi.textContent,s=r.length;for(n=0;n<t&&e[n]===r[n];n++);var o=t-n;for(i=1;i<=o&&e[t-i]===r[s-i];i++);return bl=r.slice(n,1<i?1-i:void 0)}function Pl(n){var e=n.keyCode;return"charCode"in n?(n=n.charCode,n===0&&e===13&&(n=13)):n=e,n===10&&(n=13),32<=n||n===13?n:0}function Oa(){return!0}function Cp(){return!1}function En(n){function e(t,i,r,s,o){this._reactName=t,this._targetInst=r,this.type=i,this.nativeEvent=s,this.target=o,this.currentTarget=null;for(var a in n)n.hasOwnProperty(a)&&(t=n[a],this[a]=t?t(s):s[a]);return this.isDefaultPrevented=(s.defaultPrevented!=null?s.defaultPrevented:s.returnValue===!1)?Oa:Cp,this.isPropagationStopped=Cp,this}return mt(e.prototype,{preventDefault:function(){this.defaultPrevented=!0;var t=this.nativeEvent;t&&(t.preventDefault?t.preventDefault():typeof t.returnValue!="unknown"&&(t.returnValue=!1),this.isDefaultPrevented=Oa)},stopPropagation:function(){var t=this.nativeEvent;t&&(t.stopPropagation?t.stopPropagation():typeof t.cancelBubble!="unknown"&&(t.cancelBubble=!0),this.isPropagationStopped=Oa)},persist:function(){},isPersistent:Oa}),e}var io={eventPhase:0,bubbles:0,cancelable:0,timeStamp:function(n){return n.timeStamp||Date.now()},defaultPrevented:0,isTrusted:0},rh=En(io),xa=mt({},io,{view:0,detail:0}),dx=En(xa),tu,nu,mo,Ac=mt({},xa,{screenX:0,screenY:0,clientX:0,clientY:0,pageX:0,pageY:0,ctrlKey:0,shiftKey:0,altKey:0,metaKey:0,getModifierState:sh,button:0,buttons:0,relatedTarget:function(n){return n.relatedTarget===void 0?n.fromElement===n.srcElement?n.toElement:n.fromElement:n.relatedTarget},movementX:function(n){return"movementX"in n?n.movementX:(n!==mo&&(mo&&n.type==="mousemove"?(tu=n.screenX-mo.screenX,nu=n.screenY-mo.screenY):nu=tu=0,mo=n),tu)},movementY:function(n){return"movementY"in n?n.movementY:nu}}),bp=En(Ac),fx=mt({},Ac,{dataTransfer:0}),hx=En(fx),px=mt({},xa,{relatedTarget:0}),iu=En(px),mx=mt({},io,{animationName:0,elapsedTime:0,pseudoElement:0}),gx=En(mx),_x=mt({},io,{clipboardData:function(n){return"clipboardData"in n?n.clipboardData:window.clipboardData}}),vx=En(_x),yx=mt({},io,{data:0}),Pp=En(yx),xx={Esc:"Escape",Spacebar:" ",Left:"ArrowLeft",Up:"ArrowUp",Right:"ArrowRight",Down:"ArrowDown",Del:"Delete",Win:"OS",Menu:"ContextMenu",Apps:"ContextMenu",Scroll:"ScrollLock",MozPrintableKey:"Unidentified"},Sx={8:"Backspace",9:"Tab",12:"Clear",13:"Enter",16:"Shift",17:"Control",18:"Alt",19:"Pause",20:"CapsLock",27:"Escape",32:" ",33:"PageUp",34:"PageDown",35:"End",36:"Home",37:"ArrowLeft",38:"ArrowUp",39:"ArrowRight",40:"ArrowDown",45:"Insert",46:"Delete",112:"F1",113:"F2",114:"F3",115:"F4",116:"F5",117:"F6",118:"F7",119:"F8",120:"F9",121:"F10",122:"F11",123:"F12",144:"NumLock",145:"ScrollLock",224:"Meta"},Mx={Alt:"altKey",Control:"ctrlKey",Meta:"metaKey",Shift:"shiftKey"};function Ex(n){var e=this.nativeEvent;return e.getModifierState?e.getModifierState(n):(n=Mx[n])?!!e[n]:!1}function sh(){return Ex}var Tx=mt({},xa,{key:function(n){if(n.key){var e=xx[n.key]||n.key;if(e!=="Unidentified")return e}return n.type==="keypress"?(n=Pl(n),n===13?"Enter":String.fromCharCode(n)):n.type==="keydown"||n.type==="keyup"?Sx[n.keyCode]||"Unidentified":""},code:0,location:0,ctrlKey:0,shiftKey:0,altKey:0,metaKey:0,repeat:0,locale:0,getModifierState:sh,charCode:function(n){return n.type==="keypress"?Pl(n):0},keyCode:function(n){return n.type==="keydown"||n.type==="keyup"?n.keyCode:0},which:function(n){return n.type==="keypress"?Pl(n):n.type==="keydown"||n.type==="keyup"?n.keyCode:0}}),wx=En(Tx),Ax=mt({},Ac,{pointerId:0,width:0,height:0,pressure:0,tangentialPressure:0,tiltX:0,tiltY:0,twist:0,pointerType:0,isPrimary:0}),Lp=En(Ax),Rx=mt({},xa,{touches:0,targetTouches:0,changedTouches:0,altKey:0,metaKey:0,ctrlKey:0,shiftKey:0,getModifierState:sh}),Cx=En(Rx),bx=mt({},io,{propertyName:0,elapsedTime:0,pseudoElement:0}),Px=En(bx),Lx=mt({},Ac,{deltaX:function(n){return"deltaX"in n?n.deltaX:"wheelDeltaX"in n?-n.wheelDeltaX:0},deltaY:function(n){return"deltaY"in n?n.deltaY:"wheelDeltaY"in n?-n.wheelDeltaY:"wheelDelta"in n?-n.wheelDelta:0},deltaZ:0,deltaMode:0}),Ix=En(Lx),Nx=[9,13,27,32],oh=Ci&&"CompositionEvent"in window,zo=null;Ci&&"documentMode"in document&&(zo=document.documentMode);var Dx=Ci&&"TextEvent"in window&&!zo,$_=Ci&&(!oh||zo&&8<zo&&11>=zo),Ip=" ",Np=!1;function Z_(n,e){switch(n){case"keyup":return Nx.indexOf(e.keyCode)!==-1;case"keydown":return e.keyCode!==229;case"keypress":case"mousedown":case"focusout":return!0;default:return!1}}function Q_(n){return n=n.detail,typeof n=="object"&&"data"in n?n.data:null}var _s=!1;function Ux(n,e){switch(n){case"compositionend":return Q_(e);case"keypress":return e.which!==32?null:(Np=!0,Ip);case"textInput":return n=e.data,n===Ip&&Np?null:n;default:return null}}function Fx(n,e){if(_s)return n==="compositionend"||!oh&&Z_(n,e)?(n=q_(),bl=ih=Zi=null,_s=!1,n):null;switch(n){case"paste":return null;case"keypress":if(!(e.ctrlKey||e.altKey||e.metaKey)||e.ctrlKey&&e.altKey){if(e.char&&1<e.char.length)return e.char;if(e.which)return String.fromCharCode(e.which)}return null;case"compositionend":return $_&&e.locale!=="ko"?null:e.data;default:return null}}var Ox={color:!0,date:!0,datetime:!0,"datetime-local":!0,email:!0,month:!0,number:!0,password:!0,range:!0,search:!0,tel:!0,text:!0,time:!0,url:!0,week:!0};function Dp(n){var e=n&&n.nodeName&&n.nodeName.toLowerCase();return e==="input"?!!Ox[n.type]:e==="textarea"}function J_(n,e,t,i){P_(i),e=Ql(e,"onChange"),0<e.length&&(t=new rh("onChange","change",null,t,i),n.push({event:t,listeners:e}))}var Ho=null,na=null;function kx(n){u0(n,0)}function Rc(n){var e=xs(n);if(E_(e))return n}function Bx(n,e){if(n==="change")return e}var e0=!1;if(Ci){var ru;if(Ci){var su="oninput"in document;if(!su){var Up=document.createElement("div");Up.setAttribute("oninput","return;"),su=typeof Up.oninput=="function"}ru=su}else ru=!1;e0=ru&&(!document.documentMode||9<document.documentMode)}function Fp(){Ho&&(Ho.detachEvent("onpropertychange",t0),na=Ho=null)}function t0(n){if(n.propertyName==="value"&&Rc(na)){var e=[];J_(e,na,n,Qf(n)),D_(kx,e)}}function zx(n,e,t){n==="focusin"?(Fp(),Ho=e,na=t,Ho.attachEvent("onpropertychange",t0)):n==="focusout"&&Fp()}function Hx(n){if(n==="selectionchange"||n==="keyup"||n==="keydown")return Rc(na)}function Vx(n,e){if(n==="click")return Rc(e)}function Gx(n,e){if(n==="input"||n==="change")return Rc(e)}function Wx(n,e){return n===e&&(n!==0||1/n===1/e)||n!==n&&e!==e}var $n=typeof Object.is=="function"?Object.is:Wx;function ia(n,e){if($n(n,e))return!0;if(typeof n!="object"||n===null||typeof e!="object"||e===null)return!1;var t=Object.keys(n),i=Object.keys(e);if(t.length!==i.length)return!1;for(i=0;i<t.length;i++){var r=t[i];if(!ud.call(e,r)||!$n(n[r],e[r]))return!1}return!0}function Op(n){for(;n&&n.firstChild;)n=n.firstChild;return n}function kp(n,e){var t=Op(n);n=0;for(var i;t;){if(t.nodeType===3){if(i=n+t.textContent.length,n<=e&&i>=e)return{node:t,offset:e-n};n=i}e:{for(;t;){if(t.nextSibling){t=t.nextSibling;break e}t=t.parentNode}t=void 0}t=Op(t)}}function n0(n,e){return n&&e?n===e?!0:n&&n.nodeType===3?!1:e&&e.nodeType===3?n0(n,e.parentNode):"contains"in n?n.contains(e):n.compareDocumentPosition?!!(n.compareDocumentPosition(e)&16):!1:!1}function i0(){for(var n=window,e=Xl();e instanceof n.HTMLIFrameElement;){try{var t=typeof e.contentWindow.location.href=="string"}catch{t=!1}if(t)n=e.contentWindow;else break;e=Xl(n.document)}return e}function ah(n){var e=n&&n.nodeName&&n.nodeName.toLowerCase();return e&&(e==="input"&&(n.type==="text"||n.type==="search"||n.type==="tel"||n.type==="url"||n.type==="password")||e==="textarea"||n.contentEditable==="true")}function Xx(n){var e=i0(),t=n.focusedElem,i=n.selectionRange;if(e!==t&&t&&t.ownerDocument&&n0(t.ownerDocument.documentElement,t)){if(i!==null&&ah(t)){if(e=i.start,n=i.end,n===void 0&&(n=e),"selectionStart"in t)t.selectionStart=e,t.selectionEnd=Math.min(n,t.value.length);else if(n=(e=t.ownerDocument||document)&&e.defaultView||window,n.getSelection){n=n.getSelection();var r=t.textContent.length,s=Math.min(i.start,r);i=i.end===void 0?s:Math.min(i.end,r),!n.extend&&s>i&&(r=i,i=s,s=r),r=kp(t,s);var o=kp(t,i);r&&o&&(n.rangeCount!==1||n.anchorNode!==r.node||n.anchorOffset!==r.offset||n.focusNode!==o.node||n.focusOffset!==o.offset)&&(e=e.createRange(),e.setStart(r.node,r.offset),n.removeAllRanges(),s>i?(n.addRange(e),n.extend(o.node,o.offset)):(e.setEnd(o.node,o.offset),n.addRange(e)))}}for(e=[],n=t;n=n.parentNode;)n.nodeType===1&&e.push({element:n,left:n.scrollLeft,top:n.scrollTop});for(typeof t.focus=="function"&&t.focus(),t=0;t<e.length;t++)n=e[t],n.element.scrollLeft=n.left,n.element.scrollTop=n.top}}var jx=Ci&&"documentMode"in document&&11>=document.documentMode,vs=null,bd=null,Vo=null,Pd=!1;function Bp(n,e,t){var i=t.window===t?t.document:t.nodeType===9?t:t.ownerDocument;Pd||vs==null||vs!==Xl(i)||(i=vs,"selectionStart"in i&&ah(i)?i={start:i.selectionStart,end:i.selectionEnd}:(i=(i.ownerDocument&&i.ownerDocument.defaultView||window).getSelection(),i={anchorNode:i.anchorNode,anchorOffset:i.anchorOffset,focusNode:i.focusNode,focusOffset:i.focusOffset}),Vo&&ia(Vo,i)||(Vo=i,i=Ql(bd,"onSelect"),0<i.length&&(e=new rh("onSelect","select",null,e,t),n.push({event:e,listeners:i}),e.target=vs)))}function ka(n,e){var t={};return t[n.toLowerCase()]=e.toLowerCase(),t["Webkit"+n]="webkit"+e,t["Moz"+n]="moz"+e,t}var ys={animationend:ka("Animation","AnimationEnd"),animationiteration:ka("Animation","AnimationIteration"),animationstart:ka("Animation","AnimationStart"),transitionend:ka("Transition","TransitionEnd")},ou={},r0={};Ci&&(r0=document.createElement("div").style,"AnimationEvent"in window||(delete ys.animationend.animation,delete ys.animationiteration.animation,delete ys.animationstart.animation),"TransitionEvent"in window||delete ys.transitionend.transition);function Cc(n){if(ou[n])return ou[n];if(!ys[n])return n;var e=ys[n],t;for(t in e)if(e.hasOwnProperty(t)&&t in r0)return ou[n]=e[t];return n}var s0=Cc("animationend"),o0=Cc("animationiteration"),a0=Cc("animationstart"),l0=Cc("transitionend"),c0=new Map,zp="abort auxClick cancel canPlay canPlayThrough click close contextMenu copy cut drag dragEnd dragEnter dragExit dragLeave dragOver dragStart drop durationChange emptied encrypted ended error gotPointerCapture input invalid keyDown keyPress keyUp load loadedData loadedMetadata loadStart lostPointerCapture mouseDown mouseMove mouseOut mouseOver mouseUp paste pause play playing pointerCancel pointerDown pointerMove pointerOut pointerOver pointerUp progress rateChange reset resize seeked seeking stalled submit suspend timeUpdate touchCancel touchEnd touchStart volumeChange scroll toggle touchMove waiting wheel".split(" ");function hr(n,e){c0.set(n,e),Gr(e,[n])}for(var au=0;au<zp.length;au++){var lu=zp[au],Yx=lu.toLowerCase(),Kx=lu[0].toUpperCase()+lu.slice(1);hr(Yx,"on"+Kx)}hr(s0,"onAnimationEnd");hr(o0,"onAnimationIteration");hr(a0,"onAnimationStart");hr("dblclick","onDoubleClick");hr("focusin","onFocus");hr("focusout","onBlur");hr(l0,"onTransitionEnd");Bs("onMouseEnter",["mouseout","mouseover"]);Bs("onMouseLeave",["mouseout","mouseover"]);Bs("onPointerEnter",["pointerout","pointerover"]);Bs("onPointerLeave",["pointerout","pointerover"]);Gr("onChange","change click focusin focusout input keydown keyup selectionchange".split(" "));Gr("onSelect","focusout contextmenu dragend focusin keydown keyup mousedown mouseup selectionchange".split(" "));Gr("onBeforeInput",["compositionend","keypress","textInput","paste"]);Gr("onCompositionEnd","compositionend focusout keydown keypress keyup mousedown".split(" "));Gr("onCompositionStart","compositionstart focusout keydown keypress keyup mousedown".split(" "));Gr("onCompositionUpdate","compositionupdate focusout keydown keypress keyup mousedown".split(" "));var Uo="abort canplay canplaythrough durationchange emptied encrypted ended error loadeddata loadedmetadata loadstart pause play playing progress ratechange resize seeked seeking stalled suspend timeupdate volumechange waiting".split(" "),qx=new Set("cancel close invalid load scroll toggle".split(" ").concat(Uo));function Hp(n,e,t){var i=n.type||"unknown-event";n.currentTarget=t,Yy(i,e,void 0,n),n.currentTarget=null}function u0(n,e){e=(e&4)!==0;for(var t=0;t<n.length;t++){var i=n[t],r=i.event;i=i.listeners;e:{var s=void 0;if(e)for(var o=i.length-1;0<=o;o--){var a=i[o],l=a.instance,c=a.currentTarget;if(a=a.listener,l!==s&&r.isPropagationStopped())break e;Hp(r,a,c),s=l}else for(o=0;o<i.length;o++){if(a=i[o],l=a.instance,c=a.currentTarget,a=a.listener,l!==s&&r.isPropagationStopped())break e;Hp(r,a,c),s=l}}}if(Yl)throw n=wd,Yl=!1,wd=null,n}function at(n,e){var t=e[Ud];t===void 0&&(t=e[Ud]=new Set);var i=n+"__bubble";t.has(i)||(d0(e,n,2,!1),t.add(i))}function cu(n,e,t){var i=0;e&&(i|=4),d0(t,n,i,e)}var Ba="_reactListening"+Math.random().toString(36).slice(2);function ra(n){if(!n[Ba]){n[Ba]=!0,v_.forEach(function(t){t!=="selectionchange"&&(qx.has(t)||cu(t,!1,n),cu(t,!0,n))});var e=n.nodeType===9?n:n.ownerDocument;e===null||e[Ba]||(e[Ba]=!0,cu("selectionchange",!1,e))}}function d0(n,e,t,i){switch(K_(e)){case 1:var r=cx;break;case 4:r=ux;break;default:r=nh}t=r.bind(null,e,t,n),r=void 0,!Td||e!=="touchstart"&&e!=="touchmove"&&e!=="wheel"||(r=!0),i?r!==void 0?n.addEventListener(e,t,{capture:!0,passive:r}):n.addEventListener(e,t,!0):r!==void 0?n.addEventListener(e,t,{passive:r}):n.addEventListener(e,t,!1)}function uu(n,e,t,i,r){var s=i;if(!(e&1)&&!(e&2)&&i!==null)e:for(;;){if(i===null)return;var o=i.tag;if(o===3||o===4){var a=i.stateNode.containerInfo;if(a===r||a.nodeType===8&&a.parentNode===r)break;if(o===4)for(o=i.return;o!==null;){var l=o.tag;if((l===3||l===4)&&(l=o.stateNode.containerInfo,l===r||l.nodeType===8&&l.parentNode===r))return;o=o.return}for(;a!==null;){if(o=Lr(a),o===null)return;if(l=o.tag,l===5||l===6){i=s=o;continue e}a=a.parentNode}}i=i.return}D_(function(){var c=s,u=Qf(t),d=[];e:{var f=c0.get(n);if(f!==void 0){var p=rh,g=n;switch(n){case"keypress":if(Pl(t)===0)break e;case"keydown":case"keyup":p=wx;break;case"focusin":g="focus",p=iu;break;case"focusout":g="blur",p=iu;break;case"beforeblur":case"afterblur":p=iu;break;case"click":if(t.button===2)break e;case"auxclick":case"dblclick":case"mousedown":case"mousemove":case"mouseup":case"mouseout":case"mouseover":case"contextmenu":p=bp;break;case"drag":case"dragend":case"dragenter":case"dragexit":case"dragleave":case"dragover":case"dragstart":case"drop":p=hx;break;case"touchcancel":case"touchend":case"touchmove":case"touchstart":p=Cx;break;case s0:case o0:case a0:p=gx;break;case l0:p=Px;break;case"scroll":p=dx;break;case"wheel":p=Ix;break;case"copy":case"cut":case"paste":p=vx;break;case"gotpointercapture":case"lostpointercapture":case"pointercancel":case"pointerdown":case"pointermove":case"pointerout":case"pointerover":case"pointerup":p=Lp}var y=(e&4)!==0,m=!y&&n==="scroll",h=y?f!==null?f+"Capture":null:f;y=[];for(var _=c,v;_!==null;){v=_;var x=v.stateNode;if(v.tag===5&&x!==null&&(v=x,h!==null&&(x=Qo(_,h),x!=null&&y.push(sa(_,x,v)))),m)break;_=_.return}0<y.length&&(f=new p(f,g,null,t,u),d.push({event:f,listeners:y}))}}if(!(e&7)){e:{if(f=n==="mouseover"||n==="pointerover",p=n==="mouseout"||n==="pointerout",f&&t!==Md&&(g=t.relatedTarget||t.fromElement)&&(Lr(g)||g[bi]))break e;if((p||f)&&(f=u.window===u?u:(f=u.ownerDocument)?f.defaultView||f.parentWindow:window,p?(g=t.relatedTarget||t.toElement,p=c,g=g?Lr(g):null,g!==null&&(m=Wr(g),g!==m||g.tag!==5&&g.tag!==6)&&(g=null)):(p=null,g=c),p!==g)){if(y=bp,x="onMouseLeave",h="onMouseEnter",_="mouse",(n==="pointerout"||n==="pointerover")&&(y=Lp,x="onPointerLeave",h="onPointerEnter",_="pointer"),m=p==null?f:xs(p),v=g==null?f:xs(g),f=new y(x,_+"leave",p,t,u),f.target=m,f.relatedTarget=v,x=null,Lr(u)===c&&(y=new y(h,_+"enter",g,t,u),y.target=v,y.relatedTarget=m,x=y),m=x,p&&g)t:{for(y=p,h=g,_=0,v=y;v;v=jr(v))_++;for(v=0,x=h;x;x=jr(x))v++;for(;0<_-v;)y=jr(y),_--;for(;0<v-_;)h=jr(h),v--;for(;_--;){if(y===h||h!==null&&y===h.alternate)break t;y=jr(y),h=jr(h)}y=null}else y=null;p!==null&&Vp(d,f,p,y,!1),g!==null&&m!==null&&Vp(d,m,g,y,!0)}}e:{if(f=c?xs(c):window,p=f.nodeName&&f.nodeName.toLowerCase(),p==="select"||p==="input"&&f.type==="file")var R=Bx;else if(Dp(f))if(e0)R=Gx;else{R=Hx;var A=zx}else(p=f.nodeName)&&p.toLowerCase()==="input"&&(f.type==="checkbox"||f.type==="radio")&&(R=Vx);if(R&&(R=R(n,c))){J_(d,R,t,u);break e}A&&A(n,f,c),n==="focusout"&&(A=f._wrapperState)&&A.controlled&&f.type==="number"&&_d(f,"number",f.value)}switch(A=c?xs(c):window,n){case"focusin":(Dp(A)||A.contentEditable==="true")&&(vs=A,bd=c,Vo=null);break;case"focusout":Vo=bd=vs=null;break;case"mousedown":Pd=!0;break;case"contextmenu":case"mouseup":case"dragend":Pd=!1,Bp(d,t,u);break;case"selectionchange":if(jx)break;case"keydown":case"keyup":Bp(d,t,u)}var w;if(oh)e:{switch(n){case"compositionstart":var b="onCompositionStart";break e;case"compositionend":b="onCompositionEnd";break e;case"compositionupdate":b="onCompositionUpdate";break e}b=void 0}else _s?Z_(n,t)&&(b="onCompositionEnd"):n==="keydown"&&t.keyCode===229&&(b="onCompositionStart");b&&($_&&t.locale!=="ko"&&(_s||b!=="onCompositionStart"?b==="onCompositionEnd"&&_s&&(w=q_()):(Zi=u,ih="value"in Zi?Zi.value:Zi.textContent,_s=!0)),A=Ql(c,b),0<A.length&&(b=new Pp(b,n,null,t,u),d.push({event:b,listeners:A}),w?b.data=w:(w=Q_(t),w!==null&&(b.data=w)))),(w=Dx?Ux(n,t):Fx(n,t))&&(c=Ql(c,"onBeforeInput"),0<c.length&&(u=new Pp("onBeforeInput","beforeinput",null,t,u),d.push({event:u,listeners:c}),u.data=w))}u0(d,e)})}function sa(n,e,t){return{instance:n,listener:e,currentTarget:t}}function Ql(n,e){for(var t=e+"Capture",i=[];n!==null;){var r=n,s=r.stateNode;r.tag===5&&s!==null&&(r=s,s=Qo(n,t),s!=null&&i.unshift(sa(n,s,r)),s=Qo(n,e),s!=null&&i.push(sa(n,s,r))),n=n.return}return i}function jr(n){if(n===null)return null;do n=n.return;while(n&&n.tag!==5);return n||null}function Vp(n,e,t,i,r){for(var s=e._reactName,o=[];t!==null&&t!==i;){var a=t,l=a.alternate,c=a.stateNode;if(l!==null&&l===i)break;a.tag===5&&c!==null&&(a=c,r?(l=Qo(t,s),l!=null&&o.unshift(sa(t,l,a))):r||(l=Qo(t,s),l!=null&&o.push(sa(t,l,a)))),t=t.return}o.length!==0&&n.push({event:e,listeners:o})}var $x=/\r\n?/g,Zx=/\u0000|\uFFFD/g;function Gp(n){return(typeof n=="string"?n:""+n).replace($x,`
`).replace(Zx,"")}function za(n,e,t){if(e=Gp(e),Gp(n)!==e&&t)throw Error(te(425))}function Jl(){}var Ld=null,Id=null;function Nd(n,e){return n==="textarea"||n==="noscript"||typeof e.children=="string"||typeof e.children=="number"||typeof e.dangerouslySetInnerHTML=="object"&&e.dangerouslySetInnerHTML!==null&&e.dangerouslySetInnerHTML.__html!=null}var Dd=typeof setTimeout=="function"?setTimeout:void 0,Qx=typeof clearTimeout=="function"?clearTimeout:void 0,Wp=typeof Promise=="function"?Promise:void 0,Jx=typeof queueMicrotask=="function"?queueMicrotask:typeof Wp<"u"?function(n){return Wp.resolve(null).then(n).catch(eS)}:Dd;function eS(n){setTimeout(function(){throw n})}function du(n,e){var t=e,i=0;do{var r=t.nextSibling;if(n.removeChild(t),r&&r.nodeType===8)if(t=r.data,t==="/$"){if(i===0){n.removeChild(r),ta(e);return}i--}else t!=="$"&&t!=="$?"&&t!=="$!"||i++;t=r}while(t);ta(e)}function rr(n){for(;n!=null;n=n.nextSibling){var e=n.nodeType;if(e===1||e===3)break;if(e===8){if(e=n.data,e==="$"||e==="$!"||e==="$?")break;if(e==="/$")return null}}return n}function Xp(n){n=n.previousSibling;for(var e=0;n;){if(n.nodeType===8){var t=n.data;if(t==="$"||t==="$!"||t==="$?"){if(e===0)return n;e--}else t==="/$"&&e++}n=n.previousSibling}return null}var ro=Math.random().toString(36).slice(2),ti="__reactFiber$"+ro,oa="__reactProps$"+ro,bi="__reactContainer$"+ro,Ud="__reactEvents$"+ro,tS="__reactListeners$"+ro,nS="__reactHandles$"+ro;function Lr(n){var e=n[ti];if(e)return e;for(var t=n.parentNode;t;){if(e=t[bi]||t[ti]){if(t=e.alternate,e.child!==null||t!==null&&t.child!==null)for(n=Xp(n);n!==null;){if(t=n[ti])return t;n=Xp(n)}return e}n=t,t=n.parentNode}return null}function Sa(n){return n=n[ti]||n[bi],!n||n.tag!==5&&n.tag!==6&&n.tag!==13&&n.tag!==3?null:n}function xs(n){if(n.tag===5||n.tag===6)return n.stateNode;throw Error(te(33))}function bc(n){return n[oa]||null}var Fd=[],Ss=-1;function pr(n){return{current:n}}function ct(n){0>Ss||(n.current=Fd[Ss],Fd[Ss]=null,Ss--)}function ot(n,e){Ss++,Fd[Ss]=n.current,n.current=e}var fr={},Kt=pr(fr),cn=pr(!1),Fr=fr;function zs(n,e){var t=n.type.contextTypes;if(!t)return fr;var i=n.stateNode;if(i&&i.__reactInternalMemoizedUnmaskedChildContext===e)return i.__reactInternalMemoizedMaskedChildContext;var r={},s;for(s in t)r[s]=e[s];return i&&(n=n.stateNode,n.__reactInternalMemoizedUnmaskedChildContext=e,n.__reactInternalMemoizedMaskedChildContext=r),r}function un(n){return n=n.childContextTypes,n!=null}function ec(){ct(cn),ct(Kt)}function jp(n,e,t){if(Kt.current!==fr)throw Error(te(168));ot(Kt,e),ot(cn,t)}function f0(n,e,t){var i=n.stateNode;if(e=e.childContextTypes,typeof i.getChildContext!="function")return t;i=i.getChildContext();for(var r in i)if(!(r in e))throw Error(te(108,zy(n)||"Unknown",r));return mt({},t,i)}function tc(n){return n=(n=n.stateNode)&&n.__reactInternalMemoizedMergedChildContext||fr,Fr=Kt.current,ot(Kt,n),ot(cn,cn.current),!0}function Yp(n,e,t){var i=n.stateNode;if(!i)throw Error(te(169));t?(n=f0(n,e,Fr),i.__reactInternalMemoizedMergedChildContext=n,ct(cn),ct(Kt),ot(Kt,n)):ct(cn),ot(cn,t)}var Si=null,Pc=!1,fu=!1;function h0(n){Si===null?Si=[n]:Si.push(n)}function iS(n){Pc=!0,h0(n)}function mr(){if(!fu&&Si!==null){fu=!0;var n=0,e=it;try{var t=Si;for(it=1;n<t.length;n++){var i=t[n];do i=i(!0);while(i!==null)}Si=null,Pc=!1}catch(r){throw Si!==null&&(Si=Si.slice(n+1)),k_(Jf,mr),r}finally{it=e,fu=!1}}return null}var Ms=[],Es=0,nc=null,ic=0,Rn=[],Cn=0,Or=null,Mi=1,Ei="";function wr(n,e){Ms[Es++]=ic,Ms[Es++]=nc,nc=n,ic=e}function p0(n,e,t){Rn[Cn++]=Mi,Rn[Cn++]=Ei,Rn[Cn++]=Or,Or=n;var i=Mi;n=Ei;var r=32-jn(i)-1;i&=~(1<<r),t+=1;var s=32-jn(e)+r;if(30<s){var o=r-r%5;s=(i&(1<<o)-1).toString(32),i>>=o,r-=o,Mi=1<<32-jn(e)+r|t<<r|i,Ei=s+n}else Mi=1<<s|t<<r|i,Ei=n}function lh(n){n.return!==null&&(wr(n,1),p0(n,1,0))}function ch(n){for(;n===nc;)nc=Ms[--Es],Ms[Es]=null,ic=Ms[--Es],Ms[Es]=null;for(;n===Or;)Or=Rn[--Cn],Rn[Cn]=null,Ei=Rn[--Cn],Rn[Cn]=null,Mi=Rn[--Cn],Rn[Cn]=null}var xn=null,yn=null,ut=!1,Vn=null;function m0(n,e){var t=bn(5,null,null,0);t.elementType="DELETED",t.stateNode=e,t.return=n,e=n.deletions,e===null?(n.deletions=[t],n.flags|=16):e.push(t)}function Kp(n,e){switch(n.tag){case 5:var t=n.type;return e=e.nodeType!==1||t.toLowerCase()!==e.nodeName.toLowerCase()?null:e,e!==null?(n.stateNode=e,xn=n,yn=rr(e.firstChild),!0):!1;case 6:return e=n.pendingProps===""||e.nodeType!==3?null:e,e!==null?(n.stateNode=e,xn=n,yn=null,!0):!1;case 13:return e=e.nodeType!==8?null:e,e!==null?(t=Or!==null?{id:Mi,overflow:Ei}:null,n.memoizedState={dehydrated:e,treeContext:t,retryLane:1073741824},t=bn(18,null,null,0),t.stateNode=e,t.return=n,n.child=t,xn=n,yn=null,!0):!1;default:return!1}}function Od(n){return(n.mode&1)!==0&&(n.flags&128)===0}function kd(n){if(ut){var e=yn;if(e){var t=e;if(!Kp(n,e)){if(Od(n))throw Error(te(418));e=rr(t.nextSibling);var i=xn;e&&Kp(n,e)?m0(i,t):(n.flags=n.flags&-4097|2,ut=!1,xn=n)}}else{if(Od(n))throw Error(te(418));n.flags=n.flags&-4097|2,ut=!1,xn=n}}}function qp(n){for(n=n.return;n!==null&&n.tag!==5&&n.tag!==3&&n.tag!==13;)n=n.return;xn=n}function Ha(n){if(n!==xn)return!1;if(!ut)return qp(n),ut=!0,!1;var e;if((e=n.tag!==3)&&!(e=n.tag!==5)&&(e=n.type,e=e!=="head"&&e!=="body"&&!Nd(n.type,n.memoizedProps)),e&&(e=yn)){if(Od(n))throw g0(),Error(te(418));for(;e;)m0(n,e),e=rr(e.nextSibling)}if(qp(n),n.tag===13){if(n=n.memoizedState,n=n!==null?n.dehydrated:null,!n)throw Error(te(317));e:{for(n=n.nextSibling,e=0;n;){if(n.nodeType===8){var t=n.data;if(t==="/$"){if(e===0){yn=rr(n.nextSibling);break e}e--}else t!=="$"&&t!=="$!"&&t!=="$?"||e++}n=n.nextSibling}yn=null}}else yn=xn?rr(n.stateNode.nextSibling):null;return!0}function g0(){for(var n=yn;n;)n=rr(n.nextSibling)}function Hs(){yn=xn=null,ut=!1}function uh(n){Vn===null?Vn=[n]:Vn.push(n)}var rS=Ui.ReactCurrentBatchConfig;function go(n,e,t){if(n=t.ref,n!==null&&typeof n!="function"&&typeof n!="object"){if(t._owner){if(t=t._owner,t){if(t.tag!==1)throw Error(te(309));var i=t.stateNode}if(!i)throw Error(te(147,n));var r=i,s=""+n;return e!==null&&e.ref!==null&&typeof e.ref=="function"&&e.ref._stringRef===s?e.ref:(e=function(o){var a=r.refs;o===null?delete a[s]:a[s]=o},e._stringRef=s,e)}if(typeof n!="string")throw Error(te(284));if(!t._owner)throw Error(te(290,n))}return n}function Va(n,e){throw n=Object.prototype.toString.call(e),Error(te(31,n==="[object Object]"?"object with keys {"+Object.keys(e).join(", ")+"}":n))}function $p(n){var e=n._init;return e(n._payload)}function _0(n){function e(h,_){if(n){var v=h.deletions;v===null?(h.deletions=[_],h.flags|=16):v.push(_)}}function t(h,_){if(!n)return null;for(;_!==null;)e(h,_),_=_.sibling;return null}function i(h,_){for(h=new Map;_!==null;)_.key!==null?h.set(_.key,_):h.set(_.index,_),_=_.sibling;return h}function r(h,_){return h=lr(h,_),h.index=0,h.sibling=null,h}function s(h,_,v){return h.index=v,n?(v=h.alternate,v!==null?(v=v.index,v<_?(h.flags|=2,_):v):(h.flags|=2,_)):(h.flags|=1048576,_)}function o(h){return n&&h.alternate===null&&(h.flags|=2),h}function a(h,_,v,x){return _===null||_.tag!==6?(_=yu(v,h.mode,x),_.return=h,_):(_=r(_,v),_.return=h,_)}function l(h,_,v,x){var R=v.type;return R===gs?u(h,_,v.props.children,x,v.key):_!==null&&(_.elementType===R||typeof R=="object"&&R!==null&&R.$$typeof===ji&&$p(R)===_.type)?(x=r(_,v.props),x.ref=go(h,_,v),x.return=h,x):(x=Ol(v.type,v.key,v.props,null,h.mode,x),x.ref=go(h,_,v),x.return=h,x)}function c(h,_,v,x){return _===null||_.tag!==4||_.stateNode.containerInfo!==v.containerInfo||_.stateNode.implementation!==v.implementation?(_=xu(v,h.mode,x),_.return=h,_):(_=r(_,v.children||[]),_.return=h,_)}function u(h,_,v,x,R){return _===null||_.tag!==7?(_=Ur(v,h.mode,x,R),_.return=h,_):(_=r(_,v),_.return=h,_)}function d(h,_,v){if(typeof _=="string"&&_!==""||typeof _=="number")return _=yu(""+_,h.mode,v),_.return=h,_;if(typeof _=="object"&&_!==null){switch(_.$$typeof){case La:return v=Ol(_.type,_.key,_.props,null,h.mode,v),v.ref=go(h,null,_),v.return=h,v;case ms:return _=xu(_,h.mode,v),_.return=h,_;case ji:var x=_._init;return d(h,x(_._payload),v)}if(No(_)||uo(_))return _=Ur(_,h.mode,v,null),_.return=h,_;Va(h,_)}return null}function f(h,_,v,x){var R=_!==null?_.key:null;if(typeof v=="string"&&v!==""||typeof v=="number")return R!==null?null:a(h,_,""+v,x);if(typeof v=="object"&&v!==null){switch(v.$$typeof){case La:return v.key===R?l(h,_,v,x):null;case ms:return v.key===R?c(h,_,v,x):null;case ji:return R=v._init,f(h,_,R(v._payload),x)}if(No(v)||uo(v))return R!==null?null:u(h,_,v,x,null);Va(h,v)}return null}function p(h,_,v,x,R){if(typeof x=="string"&&x!==""||typeof x=="number")return h=h.get(v)||null,a(_,h,""+x,R);if(typeof x=="object"&&x!==null){switch(x.$$typeof){case La:return h=h.get(x.key===null?v:x.key)||null,l(_,h,x,R);case ms:return h=h.get(x.key===null?v:x.key)||null,c(_,h,x,R);case ji:var A=x._init;return p(h,_,v,A(x._payload),R)}if(No(x)||uo(x))return h=h.get(v)||null,u(_,h,x,R,null);Va(_,x)}return null}function g(h,_,v,x){for(var R=null,A=null,w=_,b=_=0,T=null;w!==null&&b<v.length;b++){w.index>b?(T=w,w=null):T=w.sibling;var M=f(h,w,v[b],x);if(M===null){w===null&&(w=T);break}n&&w&&M.alternate===null&&e(h,w),_=s(M,_,b),A===null?R=M:A.sibling=M,A=M,w=T}if(b===v.length)return t(h,w),ut&&wr(h,b),R;if(w===null){for(;b<v.length;b++)w=d(h,v[b],x),w!==null&&(_=s(w,_,b),A===null?R=w:A.sibling=w,A=w);return ut&&wr(h,b),R}for(w=i(h,w);b<v.length;b++)T=p(w,h,b,v[b],x),T!==null&&(n&&T.alternate!==null&&w.delete(T.key===null?b:T.key),_=s(T,_,b),A===null?R=T:A.sibling=T,A=T);return n&&w.forEach(function(I){return e(h,I)}),ut&&wr(h,b),R}function y(h,_,v,x){var R=uo(v);if(typeof R!="function")throw Error(te(150));if(v=R.call(v),v==null)throw Error(te(151));for(var A=R=null,w=_,b=_=0,T=null,M=v.next();w!==null&&!M.done;b++,M=v.next()){w.index>b?(T=w,w=null):T=w.sibling;var I=f(h,w,M.value,x);if(I===null){w===null&&(w=T);break}n&&w&&I.alternate===null&&e(h,w),_=s(I,_,b),A===null?R=I:A.sibling=I,A=I,w=T}if(M.done)return t(h,w),ut&&wr(h,b),R;if(w===null){for(;!M.done;b++,M=v.next())M=d(h,M.value,x),M!==null&&(_=s(M,_,b),A===null?R=M:A.sibling=M,A=M);return ut&&wr(h,b),R}for(w=i(h,w);!M.done;b++,M=v.next())M=p(w,h,b,M.value,x),M!==null&&(n&&M.alternate!==null&&w.delete(M.key===null?b:M.key),_=s(M,_,b),A===null?R=M:A.sibling=M,A=M);return n&&w.forEach(function(V){return e(h,V)}),ut&&wr(h,b),R}function m(h,_,v,x){if(typeof v=="object"&&v!==null&&v.type===gs&&v.key===null&&(v=v.props.children),typeof v=="object"&&v!==null){switch(v.$$typeof){case La:e:{for(var R=v.key,A=_;A!==null;){if(A.key===R){if(R=v.type,R===gs){if(A.tag===7){t(h,A.sibling),_=r(A,v.props.children),_.return=h,h=_;break e}}else if(A.elementType===R||typeof R=="object"&&R!==null&&R.$$typeof===ji&&$p(R)===A.type){t(h,A.sibling),_=r(A,v.props),_.ref=go(h,A,v),_.return=h,h=_;break e}t(h,A);break}else e(h,A);A=A.sibling}v.type===gs?(_=Ur(v.props.children,h.mode,x,v.key),_.return=h,h=_):(x=Ol(v.type,v.key,v.props,null,h.mode,x),x.ref=go(h,_,v),x.return=h,h=x)}return o(h);case ms:e:{for(A=v.key;_!==null;){if(_.key===A)if(_.tag===4&&_.stateNode.containerInfo===v.containerInfo&&_.stateNode.implementation===v.implementation){t(h,_.sibling),_=r(_,v.children||[]),_.return=h,h=_;break e}else{t(h,_);break}else e(h,_);_=_.sibling}_=xu(v,h.mode,x),_.return=h,h=_}return o(h);case ji:return A=v._init,m(h,_,A(v._payload),x)}if(No(v))return g(h,_,v,x);if(uo(v))return y(h,_,v,x);Va(h,v)}return typeof v=="string"&&v!==""||typeof v=="number"?(v=""+v,_!==null&&_.tag===6?(t(h,_.sibling),_=r(_,v),_.return=h,h=_):(t(h,_),_=yu(v,h.mode,x),_.return=h,h=_),o(h)):t(h,_)}return m}var Vs=_0(!0),v0=_0(!1),rc=pr(null),sc=null,Ts=null,dh=null;function fh(){dh=Ts=sc=null}function hh(n){var e=rc.current;ct(rc),n._currentValue=e}function Bd(n,e,t){for(;n!==null;){var i=n.alternate;if((n.childLanes&e)!==e?(n.childLanes|=e,i!==null&&(i.childLanes|=e)):i!==null&&(i.childLanes&e)!==e&&(i.childLanes|=e),n===t)break;n=n.return}}function Is(n,e){sc=n,dh=Ts=null,n=n.dependencies,n!==null&&n.firstContext!==null&&(n.lanes&e&&(ln=!0),n.firstContext=null)}function Nn(n){var e=n._currentValue;if(dh!==n)if(n={context:n,memoizedValue:e,next:null},Ts===null){if(sc===null)throw Error(te(308));Ts=n,sc.dependencies={lanes:0,firstContext:n}}else Ts=Ts.next=n;return e}var Ir=null;function ph(n){Ir===null?Ir=[n]:Ir.push(n)}function y0(n,e,t,i){var r=e.interleaved;return r===null?(t.next=t,ph(e)):(t.next=r.next,r.next=t),e.interleaved=t,Pi(n,i)}function Pi(n,e){n.lanes|=e;var t=n.alternate;for(t!==null&&(t.lanes|=e),t=n,n=n.return;n!==null;)n.childLanes|=e,t=n.alternate,t!==null&&(t.childLanes|=e),t=n,n=n.return;return t.tag===3?t.stateNode:null}var Yi=!1;function mh(n){n.updateQueue={baseState:n.memoizedState,firstBaseUpdate:null,lastBaseUpdate:null,shared:{pending:null,interleaved:null,lanes:0},effects:null}}function x0(n,e){n=n.updateQueue,e.updateQueue===n&&(e.updateQueue={baseState:n.baseState,firstBaseUpdate:n.firstBaseUpdate,lastBaseUpdate:n.lastBaseUpdate,shared:n.shared,effects:n.effects})}function Ri(n,e){return{eventTime:n,lane:e,tag:0,payload:null,callback:null,next:null}}function sr(n,e,t){var i=n.updateQueue;if(i===null)return null;if(i=i.shared,Ke&2){var r=i.pending;return r===null?e.next=e:(e.next=r.next,r.next=e),i.pending=e,Pi(n,t)}return r=i.interleaved,r===null?(e.next=e,ph(i)):(e.next=r.next,r.next=e),i.interleaved=e,Pi(n,t)}function Ll(n,e,t){if(e=e.updateQueue,e!==null&&(e=e.shared,(t&4194240)!==0)){var i=e.lanes;i&=n.pendingLanes,t|=i,e.lanes=t,eh(n,t)}}function Zp(n,e){var t=n.updateQueue,i=n.alternate;if(i!==null&&(i=i.updateQueue,t===i)){var r=null,s=null;if(t=t.firstBaseUpdate,t!==null){do{var o={eventTime:t.eventTime,lane:t.lane,tag:t.tag,payload:t.payload,callback:t.callback,next:null};s===null?r=s=o:s=s.next=o,t=t.next}while(t!==null);s===null?r=s=e:s=s.next=e}else r=s=e;t={baseState:i.baseState,firstBaseUpdate:r,lastBaseUpdate:s,shared:i.shared,effects:i.effects},n.updateQueue=t;return}n=t.lastBaseUpdate,n===null?t.firstBaseUpdate=e:n.next=e,t.lastBaseUpdate=e}function oc(n,e,t,i){var r=n.updateQueue;Yi=!1;var s=r.firstBaseUpdate,o=r.lastBaseUpdate,a=r.shared.pending;if(a!==null){r.shared.pending=null;var l=a,c=l.next;l.next=null,o===null?s=c:o.next=c,o=l;var u=n.alternate;u!==null&&(u=u.updateQueue,a=u.lastBaseUpdate,a!==o&&(a===null?u.firstBaseUpdate=c:a.next=c,u.lastBaseUpdate=l))}if(s!==null){var d=r.baseState;o=0,u=c=l=null,a=s;do{var f=a.lane,p=a.eventTime;if((i&f)===f){u!==null&&(u=u.next={eventTime:p,lane:0,tag:a.tag,payload:a.payload,callback:a.callback,next:null});e:{var g=n,y=a;switch(f=e,p=t,y.tag){case 1:if(g=y.payload,typeof g=="function"){d=g.call(p,d,f);break e}d=g;break e;case 3:g.flags=g.flags&-65537|128;case 0:if(g=y.payload,f=typeof g=="function"?g.call(p,d,f):g,f==null)break e;d=mt({},d,f);break e;case 2:Yi=!0}}a.callback!==null&&a.lane!==0&&(n.flags|=64,f=r.effects,f===null?r.effects=[a]:f.push(a))}else p={eventTime:p,lane:f,tag:a.tag,payload:a.payload,callback:a.callback,next:null},u===null?(c=u=p,l=d):u=u.next=p,o|=f;if(a=a.next,a===null){if(a=r.shared.pending,a===null)break;f=a,a=f.next,f.next=null,r.lastBaseUpdate=f,r.shared.pending=null}}while(!0);if(u===null&&(l=d),r.baseState=l,r.firstBaseUpdate=c,r.lastBaseUpdate=u,e=r.shared.interleaved,e!==null){r=e;do o|=r.lane,r=r.next;while(r!==e)}else s===null&&(r.shared.lanes=0);Br|=o,n.lanes=o,n.memoizedState=d}}function Qp(n,e,t){if(n=e.effects,e.effects=null,n!==null)for(e=0;e<n.length;e++){var i=n[e],r=i.callback;if(r!==null){if(i.callback=null,i=t,typeof r!="function")throw Error(te(191,r));r.call(i)}}}var Ma={},oi=pr(Ma),aa=pr(Ma),la=pr(Ma);function Nr(n){if(n===Ma)throw Error(te(174));return n}function gh(n,e){switch(ot(la,e),ot(aa,n),ot(oi,Ma),n=e.nodeType,n){case 9:case 11:e=(e=e.documentElement)?e.namespaceURI:yd(null,"");break;default:n=n===8?e.parentNode:e,e=n.namespaceURI||null,n=n.tagName,e=yd(e,n)}ct(oi),ot(oi,e)}function Gs(){ct(oi),ct(aa),ct(la)}function S0(n){Nr(la.current);var e=Nr(oi.current),t=yd(e,n.type);e!==t&&(ot(aa,n),ot(oi,t))}function _h(n){aa.current===n&&(ct(oi),ct(aa))}var ht=pr(0);function ac(n){for(var e=n;e!==null;){if(e.tag===13){var t=e.memoizedState;if(t!==null&&(t=t.dehydrated,t===null||t.data==="$?"||t.data==="$!"))return e}else if(e.tag===19&&e.memoizedProps.revealOrder!==void 0){if(e.flags&128)return e}else if(e.child!==null){e.child.return=e,e=e.child;continue}if(e===n)break;for(;e.sibling===null;){if(e.return===null||e.return===n)return null;e=e.return}e.sibling.return=e.return,e=e.sibling}return null}var hu=[];function vh(){for(var n=0;n<hu.length;n++)hu[n]._workInProgressVersionPrimary=null;hu.length=0}var Il=Ui.ReactCurrentDispatcher,pu=Ui.ReactCurrentBatchConfig,kr=0,pt=null,Ct=null,Dt=null,lc=!1,Go=!1,ca=0,sS=0;function Gt(){throw Error(te(321))}function yh(n,e){if(e===null)return!1;for(var t=0;t<e.length&&t<n.length;t++)if(!$n(n[t],e[t]))return!1;return!0}function xh(n,e,t,i,r,s){if(kr=s,pt=e,e.memoizedState=null,e.updateQueue=null,e.lanes=0,Il.current=n===null||n.memoizedState===null?cS:uS,n=t(i,r),Go){s=0;do{if(Go=!1,ca=0,25<=s)throw Error(te(301));s+=1,Dt=Ct=null,e.updateQueue=null,Il.current=dS,n=t(i,r)}while(Go)}if(Il.current=cc,e=Ct!==null&&Ct.next!==null,kr=0,Dt=Ct=pt=null,lc=!1,e)throw Error(te(300));return n}function Sh(){var n=ca!==0;return ca=0,n}function Jn(){var n={memoizedState:null,baseState:null,baseQueue:null,queue:null,next:null};return Dt===null?pt.memoizedState=Dt=n:Dt=Dt.next=n,Dt}function Dn(){if(Ct===null){var n=pt.alternate;n=n!==null?n.memoizedState:null}else n=Ct.next;var e=Dt===null?pt.memoizedState:Dt.next;if(e!==null)Dt=e,Ct=n;else{if(n===null)throw Error(te(310));Ct=n,n={memoizedState:Ct.memoizedState,baseState:Ct.baseState,baseQueue:Ct.baseQueue,queue:Ct.queue,next:null},Dt===null?pt.memoizedState=Dt=n:Dt=Dt.next=n}return Dt}function ua(n,e){return typeof e=="function"?e(n):e}function mu(n){var e=Dn(),t=e.queue;if(t===null)throw Error(te(311));t.lastRenderedReducer=n;var i=Ct,r=i.baseQueue,s=t.pending;if(s!==null){if(r!==null){var o=r.next;r.next=s.next,s.next=o}i.baseQueue=r=s,t.pending=null}if(r!==null){s=r.next,i=i.baseState;var a=o=null,l=null,c=s;do{var u=c.lane;if((kr&u)===u)l!==null&&(l=l.next={lane:0,action:c.action,hasEagerState:c.hasEagerState,eagerState:c.eagerState,next:null}),i=c.hasEagerState?c.eagerState:n(i,c.action);else{var d={lane:u,action:c.action,hasEagerState:c.hasEagerState,eagerState:c.eagerState,next:null};l===null?(a=l=d,o=i):l=l.next=d,pt.lanes|=u,Br|=u}c=c.next}while(c!==null&&c!==s);l===null?o=i:l.next=a,$n(i,e.memoizedState)||(ln=!0),e.memoizedState=i,e.baseState=o,e.baseQueue=l,t.lastRenderedState=i}if(n=t.interleaved,n!==null){r=n;do s=r.lane,pt.lanes|=s,Br|=s,r=r.next;while(r!==n)}else r===null&&(t.lanes=0);return[e.memoizedState,t.dispatch]}function gu(n){var e=Dn(),t=e.queue;if(t===null)throw Error(te(311));t.lastRenderedReducer=n;var i=t.dispatch,r=t.pending,s=e.memoizedState;if(r!==null){t.pending=null;var o=r=r.next;do s=n(s,o.action),o=o.next;while(o!==r);$n(s,e.memoizedState)||(ln=!0),e.memoizedState=s,e.baseQueue===null&&(e.baseState=s),t.lastRenderedState=s}return[s,i]}function M0(){}function E0(n,e){var t=pt,i=Dn(),r=e(),s=!$n(i.memoizedState,r);if(s&&(i.memoizedState=r,ln=!0),i=i.queue,Mh(A0.bind(null,t,i,n),[n]),i.getSnapshot!==e||s||Dt!==null&&Dt.memoizedState.tag&1){if(t.flags|=2048,da(9,w0.bind(null,t,i,r,e),void 0,null),Ft===null)throw Error(te(349));kr&30||T0(t,e,r)}return r}function T0(n,e,t){n.flags|=16384,n={getSnapshot:e,value:t},e=pt.updateQueue,e===null?(e={lastEffect:null,stores:null},pt.updateQueue=e,e.stores=[n]):(t=e.stores,t===null?e.stores=[n]:t.push(n))}function w0(n,e,t,i){e.value=t,e.getSnapshot=i,R0(e)&&C0(n)}function A0(n,e,t){return t(function(){R0(e)&&C0(n)})}function R0(n){var e=n.getSnapshot;n=n.value;try{var t=e();return!$n(n,t)}catch{return!0}}function C0(n){var e=Pi(n,1);e!==null&&Yn(e,n,1,-1)}function Jp(n){var e=Jn();return typeof n=="function"&&(n=n()),e.memoizedState=e.baseState=n,n={pending:null,interleaved:null,lanes:0,dispatch:null,lastRenderedReducer:ua,lastRenderedState:n},e.queue=n,n=n.dispatch=lS.bind(null,pt,n),[e.memoizedState,n]}function da(n,e,t,i){return n={tag:n,create:e,destroy:t,deps:i,next:null},e=pt.updateQueue,e===null?(e={lastEffect:null,stores:null},pt.updateQueue=e,e.lastEffect=n.next=n):(t=e.lastEffect,t===null?e.lastEffect=n.next=n:(i=t.next,t.next=n,n.next=i,e.lastEffect=n)),n}function b0(){return Dn().memoizedState}function Nl(n,e,t,i){var r=Jn();pt.flags|=n,r.memoizedState=da(1|e,t,void 0,i===void 0?null:i)}function Lc(n,e,t,i){var r=Dn();i=i===void 0?null:i;var s=void 0;if(Ct!==null){var o=Ct.memoizedState;if(s=o.destroy,i!==null&&yh(i,o.deps)){r.memoizedState=da(e,t,s,i);return}}pt.flags|=n,r.memoizedState=da(1|e,t,s,i)}function em(n,e){return Nl(8390656,8,n,e)}function Mh(n,e){return Lc(2048,8,n,e)}function P0(n,e){return Lc(4,2,n,e)}function L0(n,e){return Lc(4,4,n,e)}function I0(n,e){if(typeof e=="function")return n=n(),e(n),function(){e(null)};if(e!=null)return n=n(),e.current=n,function(){e.current=null}}function N0(n,e,t){return t=t!=null?t.concat([n]):null,Lc(4,4,I0.bind(null,e,n),t)}function Eh(){}function D0(n,e){var t=Dn();e=e===void 0?null:e;var i=t.memoizedState;return i!==null&&e!==null&&yh(e,i[1])?i[0]:(t.memoizedState=[n,e],n)}function U0(n,e){var t=Dn();e=e===void 0?null:e;var i=t.memoizedState;return i!==null&&e!==null&&yh(e,i[1])?i[0]:(n=n(),t.memoizedState=[n,e],n)}function F0(n,e,t){return kr&21?($n(t,e)||(t=H_(),pt.lanes|=t,Br|=t,n.baseState=!0),e):(n.baseState&&(n.baseState=!1,ln=!0),n.memoizedState=t)}function oS(n,e){var t=it;it=t!==0&&4>t?t:4,n(!0);var i=pu.transition;pu.transition={};try{n(!1),e()}finally{it=t,pu.transition=i}}function O0(){return Dn().memoizedState}function aS(n,e,t){var i=ar(n);if(t={lane:i,action:t,hasEagerState:!1,eagerState:null,next:null},k0(n))B0(e,t);else if(t=y0(n,e,t,i),t!==null){var r=nn();Yn(t,n,i,r),z0(t,e,i)}}function lS(n,e,t){var i=ar(n),r={lane:i,action:t,hasEagerState:!1,eagerState:null,next:null};if(k0(n))B0(e,r);else{var s=n.alternate;if(n.lanes===0&&(s===null||s.lanes===0)&&(s=e.lastRenderedReducer,s!==null))try{var o=e.lastRenderedState,a=s(o,t);if(r.hasEagerState=!0,r.eagerState=a,$n(a,o)){var l=e.interleaved;l===null?(r.next=r,ph(e)):(r.next=l.next,l.next=r),e.interleaved=r;return}}catch{}finally{}t=y0(n,e,r,i),t!==null&&(r=nn(),Yn(t,n,i,r),z0(t,e,i))}}function k0(n){var e=n.alternate;return n===pt||e!==null&&e===pt}function B0(n,e){Go=lc=!0;var t=n.pending;t===null?e.next=e:(e.next=t.next,t.next=e),n.pending=e}function z0(n,e,t){if(t&4194240){var i=e.lanes;i&=n.pendingLanes,t|=i,e.lanes=t,eh(n,t)}}var cc={readContext:Nn,useCallback:Gt,useContext:Gt,useEffect:Gt,useImperativeHandle:Gt,useInsertionEffect:Gt,useLayoutEffect:Gt,useMemo:Gt,useReducer:Gt,useRef:Gt,useState:Gt,useDebugValue:Gt,useDeferredValue:Gt,useTransition:Gt,useMutableSource:Gt,useSyncExternalStore:Gt,useId:Gt,unstable_isNewReconciler:!1},cS={readContext:Nn,useCallback:function(n,e){return Jn().memoizedState=[n,e===void 0?null:e],n},useContext:Nn,useEffect:em,useImperativeHandle:function(n,e,t){return t=t!=null?t.concat([n]):null,Nl(4194308,4,I0.bind(null,e,n),t)},useLayoutEffect:function(n,e){return Nl(4194308,4,n,e)},useInsertionEffect:function(n,e){return Nl(4,2,n,e)},useMemo:function(n,e){var t=Jn();return e=e===void 0?null:e,n=n(),t.memoizedState=[n,e],n},useReducer:function(n,e,t){var i=Jn();return e=t!==void 0?t(e):e,i.memoizedState=i.baseState=e,n={pending:null,interleaved:null,lanes:0,dispatch:null,lastRenderedReducer:n,lastRenderedState:e},i.queue=n,n=n.dispatch=aS.bind(null,pt,n),[i.memoizedState,n]},useRef:function(n){var e=Jn();return n={current:n},e.memoizedState=n},useState:Jp,useDebugValue:Eh,useDeferredValue:function(n){return Jn().memoizedState=n},useTransition:function(){var n=Jp(!1),e=n[0];return n=oS.bind(null,n[1]),Jn().memoizedState=n,[e,n]},useMutableSource:function(){},useSyncExternalStore:function(n,e,t){var i=pt,r=Jn();if(ut){if(t===void 0)throw Error(te(407));t=t()}else{if(t=e(),Ft===null)throw Error(te(349));kr&30||T0(i,e,t)}r.memoizedState=t;var s={value:t,getSnapshot:e};return r.queue=s,em(A0.bind(null,i,s,n),[n]),i.flags|=2048,da(9,w0.bind(null,i,s,t,e),void 0,null),t},useId:function(){var n=Jn(),e=Ft.identifierPrefix;if(ut){var t=Ei,i=Mi;t=(i&~(1<<32-jn(i)-1)).toString(32)+t,e=":"+e+"R"+t,t=ca++,0<t&&(e+="H"+t.toString(32)),e+=":"}else t=sS++,e=":"+e+"r"+t.toString(32)+":";return n.memoizedState=e},unstable_isNewReconciler:!1},uS={readContext:Nn,useCallback:D0,useContext:Nn,useEffect:Mh,useImperativeHandle:N0,useInsertionEffect:P0,useLayoutEffect:L0,useMemo:U0,useReducer:mu,useRef:b0,useState:function(){return mu(ua)},useDebugValue:Eh,useDeferredValue:function(n){var e=Dn();return F0(e,Ct.memoizedState,n)},useTransition:function(){var n=mu(ua)[0],e=Dn().memoizedState;return[n,e]},useMutableSource:M0,useSyncExternalStore:E0,useId:O0,unstable_isNewReconciler:!1},dS={readContext:Nn,useCallback:D0,useContext:Nn,useEffect:Mh,useImperativeHandle:N0,useInsertionEffect:P0,useLayoutEffect:L0,useMemo:U0,useReducer:gu,useRef:b0,useState:function(){return gu(ua)},useDebugValue:Eh,useDeferredValue:function(n){var e=Dn();return Ct===null?e.memoizedState=n:F0(e,Ct.memoizedState,n)},useTransition:function(){var n=gu(ua)[0],e=Dn().memoizedState;return[n,e]},useMutableSource:M0,useSyncExternalStore:E0,useId:O0,unstable_isNewReconciler:!1};function zn(n,e){if(n&&n.defaultProps){e=mt({},e),n=n.defaultProps;for(var t in n)e[t]===void 0&&(e[t]=n[t]);return e}return e}function zd(n,e,t,i){e=n.memoizedState,t=t(i,e),t=t==null?e:mt({},e,t),n.memoizedState=t,n.lanes===0&&(n.updateQueue.baseState=t)}var Ic={isMounted:function(n){return(n=n._reactInternals)?Wr(n)===n:!1},enqueueSetState:function(n,e,t){n=n._reactInternals;var i=nn(),r=ar(n),s=Ri(i,r);s.payload=e,t!=null&&(s.callback=t),e=sr(n,s,r),e!==null&&(Yn(e,n,r,i),Ll(e,n,r))},enqueueReplaceState:function(n,e,t){n=n._reactInternals;var i=nn(),r=ar(n),s=Ri(i,r);s.tag=1,s.payload=e,t!=null&&(s.callback=t),e=sr(n,s,r),e!==null&&(Yn(e,n,r,i),Ll(e,n,r))},enqueueForceUpdate:function(n,e){n=n._reactInternals;var t=nn(),i=ar(n),r=Ri(t,i);r.tag=2,e!=null&&(r.callback=e),e=sr(n,r,i),e!==null&&(Yn(e,n,i,t),Ll(e,n,i))}};function tm(n,e,t,i,r,s,o){return n=n.stateNode,typeof n.shouldComponentUpdate=="function"?n.shouldComponentUpdate(i,s,o):e.prototype&&e.prototype.isPureReactComponent?!ia(t,i)||!ia(r,s):!0}function H0(n,e,t){var i=!1,r=fr,s=e.contextType;return typeof s=="object"&&s!==null?s=Nn(s):(r=un(e)?Fr:Kt.current,i=e.contextTypes,s=(i=i!=null)?zs(n,r):fr),e=new e(t,s),n.memoizedState=e.state!==null&&e.state!==void 0?e.state:null,e.updater=Ic,n.stateNode=e,e._reactInternals=n,i&&(n=n.stateNode,n.__reactInternalMemoizedUnmaskedChildContext=r,n.__reactInternalMemoizedMaskedChildContext=s),e}function nm(n,e,t,i){n=e.state,typeof e.componentWillReceiveProps=="function"&&e.componentWillReceiveProps(t,i),typeof e.UNSAFE_componentWillReceiveProps=="function"&&e.UNSAFE_componentWillReceiveProps(t,i),e.state!==n&&Ic.enqueueReplaceState(e,e.state,null)}function Hd(n,e,t,i){var r=n.stateNode;r.props=t,r.state=n.memoizedState,r.refs={},mh(n);var s=e.contextType;typeof s=="object"&&s!==null?r.context=Nn(s):(s=un(e)?Fr:Kt.current,r.context=zs(n,s)),r.state=n.memoizedState,s=e.getDerivedStateFromProps,typeof s=="function"&&(zd(n,e,s,t),r.state=n.memoizedState),typeof e.getDerivedStateFromProps=="function"||typeof r.getSnapshotBeforeUpdate=="function"||typeof r.UNSAFE_componentWillMount!="function"&&typeof r.componentWillMount!="function"||(e=r.state,typeof r.componentWillMount=="function"&&r.componentWillMount(),typeof r.UNSAFE_componentWillMount=="function"&&r.UNSAFE_componentWillMount(),e!==r.state&&Ic.enqueueReplaceState(r,r.state,null),oc(n,t,r,i),r.state=n.memoizedState),typeof r.componentDidMount=="function"&&(n.flags|=4194308)}function Ws(n,e){try{var t="",i=e;do t+=By(i),i=i.return;while(i);var r=t}catch(s){r=`
Error generating stack: `+s.message+`
`+s.stack}return{value:n,source:e,stack:r,digest:null}}function _u(n,e,t){return{value:n,source:null,stack:t??null,digest:e??null}}function Vd(n,e){try{console.error(e.value)}catch(t){setTimeout(function(){throw t})}}var fS=typeof WeakMap=="function"?WeakMap:Map;function V0(n,e,t){t=Ri(-1,t),t.tag=3,t.payload={element:null};var i=e.value;return t.callback=function(){dc||(dc=!0,Qd=i),Vd(n,e)},t}function G0(n,e,t){t=Ri(-1,t),t.tag=3;var i=n.type.getDerivedStateFromError;if(typeof i=="function"){var r=e.value;t.payload=function(){return i(r)},t.callback=function(){Vd(n,e)}}var s=n.stateNode;return s!==null&&typeof s.componentDidCatch=="function"&&(t.callback=function(){Vd(n,e),typeof i!="function"&&(or===null?or=new Set([this]):or.add(this));var o=e.stack;this.componentDidCatch(e.value,{componentStack:o!==null?o:""})}),t}function im(n,e,t){var i=n.pingCache;if(i===null){i=n.pingCache=new fS;var r=new Set;i.set(e,r)}else r=i.get(e),r===void 0&&(r=new Set,i.set(e,r));r.has(t)||(r.add(t),n=AS.bind(null,n,e,t),e.then(n,n))}function rm(n){do{var e;if((e=n.tag===13)&&(e=n.memoizedState,e=e!==null?e.dehydrated!==null:!0),e)return n;n=n.return}while(n!==null);return null}function sm(n,e,t,i,r){return n.mode&1?(n.flags|=65536,n.lanes=r,n):(n===e?n.flags|=65536:(n.flags|=128,t.flags|=131072,t.flags&=-52805,t.tag===1&&(t.alternate===null?t.tag=17:(e=Ri(-1,1),e.tag=2,sr(t,e,1))),t.lanes|=1),n)}var hS=Ui.ReactCurrentOwner,ln=!1;function Zt(n,e,t,i){e.child=n===null?v0(e,null,t,i):Vs(e,n.child,t,i)}function om(n,e,t,i,r){t=t.render;var s=e.ref;return Is(e,r),i=xh(n,e,t,i,s,r),t=Sh(),n!==null&&!ln?(e.updateQueue=n.updateQueue,e.flags&=-2053,n.lanes&=~r,Li(n,e,r)):(ut&&t&&lh(e),e.flags|=1,Zt(n,e,i,r),e.child)}function am(n,e,t,i,r){if(n===null){var s=t.type;return typeof s=="function"&&!Lh(s)&&s.defaultProps===void 0&&t.compare===null&&t.defaultProps===void 0?(e.tag=15,e.type=s,W0(n,e,s,i,r)):(n=Ol(t.type,null,i,e,e.mode,r),n.ref=e.ref,n.return=e,e.child=n)}if(s=n.child,!(n.lanes&r)){var o=s.memoizedProps;if(t=t.compare,t=t!==null?t:ia,t(o,i)&&n.ref===e.ref)return Li(n,e,r)}return e.flags|=1,n=lr(s,i),n.ref=e.ref,n.return=e,e.child=n}function W0(n,e,t,i,r){if(n!==null){var s=n.memoizedProps;if(ia(s,i)&&n.ref===e.ref)if(ln=!1,e.pendingProps=i=s,(n.lanes&r)!==0)n.flags&131072&&(ln=!0);else return e.lanes=n.lanes,Li(n,e,r)}return Gd(n,e,t,i,r)}function X0(n,e,t){var i=e.pendingProps,r=i.children,s=n!==null?n.memoizedState:null;if(i.mode==="hidden")if(!(e.mode&1))e.memoizedState={baseLanes:0,cachePool:null,transitions:null},ot(As,vn),vn|=t;else{if(!(t&1073741824))return n=s!==null?s.baseLanes|t:t,e.lanes=e.childLanes=1073741824,e.memoizedState={baseLanes:n,cachePool:null,transitions:null},e.updateQueue=null,ot(As,vn),vn|=n,null;e.memoizedState={baseLanes:0,cachePool:null,transitions:null},i=s!==null?s.baseLanes:t,ot(As,vn),vn|=i}else s!==null?(i=s.baseLanes|t,e.memoizedState=null):i=t,ot(As,vn),vn|=i;return Zt(n,e,r,t),e.child}function j0(n,e){var t=e.ref;(n===null&&t!==null||n!==null&&n.ref!==t)&&(e.flags|=512,e.flags|=2097152)}function Gd(n,e,t,i,r){var s=un(t)?Fr:Kt.current;return s=zs(e,s),Is(e,r),t=xh(n,e,t,i,s,r),i=Sh(),n!==null&&!ln?(e.updateQueue=n.updateQueue,e.flags&=-2053,n.lanes&=~r,Li(n,e,r)):(ut&&i&&lh(e),e.flags|=1,Zt(n,e,t,r),e.child)}function lm(n,e,t,i,r){if(un(t)){var s=!0;tc(e)}else s=!1;if(Is(e,r),e.stateNode===null)Dl(n,e),H0(e,t,i),Hd(e,t,i,r),i=!0;else if(n===null){var o=e.stateNode,a=e.memoizedProps;o.props=a;var l=o.context,c=t.contextType;typeof c=="object"&&c!==null?c=Nn(c):(c=un(t)?Fr:Kt.current,c=zs(e,c));var u=t.getDerivedStateFromProps,d=typeof u=="function"||typeof o.getSnapshotBeforeUpdate=="function";d||typeof o.UNSAFE_componentWillReceiveProps!="function"&&typeof o.componentWillReceiveProps!="function"||(a!==i||l!==c)&&nm(e,o,i,c),Yi=!1;var f=e.memoizedState;o.state=f,oc(e,i,o,r),l=e.memoizedState,a!==i||f!==l||cn.current||Yi?(typeof u=="function"&&(zd(e,t,u,i),l=e.memoizedState),(a=Yi||tm(e,t,a,i,f,l,c))?(d||typeof o.UNSAFE_componentWillMount!="function"&&typeof o.componentWillMount!="function"||(typeof o.componentWillMount=="function"&&o.componentWillMount(),typeof o.UNSAFE_componentWillMount=="function"&&o.UNSAFE_componentWillMount()),typeof o.componentDidMount=="function"&&(e.flags|=4194308)):(typeof o.componentDidMount=="function"&&(e.flags|=4194308),e.memoizedProps=i,e.memoizedState=l),o.props=i,o.state=l,o.context=c,i=a):(typeof o.componentDidMount=="function"&&(e.flags|=4194308),i=!1)}else{o=e.stateNode,x0(n,e),a=e.memoizedProps,c=e.type===e.elementType?a:zn(e.type,a),o.props=c,d=e.pendingProps,f=o.context,l=t.contextType,typeof l=="object"&&l!==null?l=Nn(l):(l=un(t)?Fr:Kt.current,l=zs(e,l));var p=t.getDerivedStateFromProps;(u=typeof p=="function"||typeof o.getSnapshotBeforeUpdate=="function")||typeof o.UNSAFE_componentWillReceiveProps!="function"&&typeof o.componentWillReceiveProps!="function"||(a!==d||f!==l)&&nm(e,o,i,l),Yi=!1,f=e.memoizedState,o.state=f,oc(e,i,o,r);var g=e.memoizedState;a!==d||f!==g||cn.current||Yi?(typeof p=="function"&&(zd(e,t,p,i),g=e.memoizedState),(c=Yi||tm(e,t,c,i,f,g,l)||!1)?(u||typeof o.UNSAFE_componentWillUpdate!="function"&&typeof o.componentWillUpdate!="function"||(typeof o.componentWillUpdate=="function"&&o.componentWillUpdate(i,g,l),typeof o.UNSAFE_componentWillUpdate=="function"&&o.UNSAFE_componentWillUpdate(i,g,l)),typeof o.componentDidUpdate=="function"&&(e.flags|=4),typeof o.getSnapshotBeforeUpdate=="function"&&(e.flags|=1024)):(typeof o.componentDidUpdate!="function"||a===n.memoizedProps&&f===n.memoizedState||(e.flags|=4),typeof o.getSnapshotBeforeUpdate!="function"||a===n.memoizedProps&&f===n.memoizedState||(e.flags|=1024),e.memoizedProps=i,e.memoizedState=g),o.props=i,o.state=g,o.context=l,i=c):(typeof o.componentDidUpdate!="function"||a===n.memoizedProps&&f===n.memoizedState||(e.flags|=4),typeof o.getSnapshotBeforeUpdate!="function"||a===n.memoizedProps&&f===n.memoizedState||(e.flags|=1024),i=!1)}return Wd(n,e,t,i,s,r)}function Wd(n,e,t,i,r,s){j0(n,e);var o=(e.flags&128)!==0;if(!i&&!o)return r&&Yp(e,t,!1),Li(n,e,s);i=e.stateNode,hS.current=e;var a=o&&typeof t.getDerivedStateFromError!="function"?null:i.render();return e.flags|=1,n!==null&&o?(e.child=Vs(e,n.child,null,s),e.child=Vs(e,null,a,s)):Zt(n,e,a,s),e.memoizedState=i.state,r&&Yp(e,t,!0),e.child}function Y0(n){var e=n.stateNode;e.pendingContext?jp(n,e.pendingContext,e.pendingContext!==e.context):e.context&&jp(n,e.context,!1),gh(n,e.containerInfo)}function cm(n,e,t,i,r){return Hs(),uh(r),e.flags|=256,Zt(n,e,t,i),e.child}var Xd={dehydrated:null,treeContext:null,retryLane:0};function jd(n){return{baseLanes:n,cachePool:null,transitions:null}}function K0(n,e,t){var i=e.pendingProps,r=ht.current,s=!1,o=(e.flags&128)!==0,a;if((a=o)||(a=n!==null&&n.memoizedState===null?!1:(r&2)!==0),a?(s=!0,e.flags&=-129):(n===null||n.memoizedState!==null)&&(r|=1),ot(ht,r&1),n===null)return kd(e),n=e.memoizedState,n!==null&&(n=n.dehydrated,n!==null)?(e.mode&1?n.data==="$!"?e.lanes=8:e.lanes=1073741824:e.lanes=1,null):(o=i.children,n=i.fallback,s?(i=e.mode,s=e.child,o={mode:"hidden",children:o},!(i&1)&&s!==null?(s.childLanes=0,s.pendingProps=o):s=Uc(o,i,0,null),n=Ur(n,i,t,null),s.return=e,n.return=e,s.sibling=n,e.child=s,e.child.memoizedState=jd(t),e.memoizedState=Xd,n):Th(e,o));if(r=n.memoizedState,r!==null&&(a=r.dehydrated,a!==null))return pS(n,e,o,i,a,r,t);if(s){s=i.fallback,o=e.mode,r=n.child,a=r.sibling;var l={mode:"hidden",children:i.children};return!(o&1)&&e.child!==r?(i=e.child,i.childLanes=0,i.pendingProps=l,e.deletions=null):(i=lr(r,l),i.subtreeFlags=r.subtreeFlags&14680064),a!==null?s=lr(a,s):(s=Ur(s,o,t,null),s.flags|=2),s.return=e,i.return=e,i.sibling=s,e.child=i,i=s,s=e.child,o=n.child.memoizedState,o=o===null?jd(t):{baseLanes:o.baseLanes|t,cachePool:null,transitions:o.transitions},s.memoizedState=o,s.childLanes=n.childLanes&~t,e.memoizedState=Xd,i}return s=n.child,n=s.sibling,i=lr(s,{mode:"visible",children:i.children}),!(e.mode&1)&&(i.lanes=t),i.return=e,i.sibling=null,n!==null&&(t=e.deletions,t===null?(e.deletions=[n],e.flags|=16):t.push(n)),e.child=i,e.memoizedState=null,i}function Th(n,e){return e=Uc({mode:"visible",children:e},n.mode,0,null),e.return=n,n.child=e}function Ga(n,e,t,i){return i!==null&&uh(i),Vs(e,n.child,null,t),n=Th(e,e.pendingProps.children),n.flags|=2,e.memoizedState=null,n}function pS(n,e,t,i,r,s,o){if(t)return e.flags&256?(e.flags&=-257,i=_u(Error(te(422))),Ga(n,e,o,i)):e.memoizedState!==null?(e.child=n.child,e.flags|=128,null):(s=i.fallback,r=e.mode,i=Uc({mode:"visible",children:i.children},r,0,null),s=Ur(s,r,o,null),s.flags|=2,i.return=e,s.return=e,i.sibling=s,e.child=i,e.mode&1&&Vs(e,n.child,null,o),e.child.memoizedState=jd(o),e.memoizedState=Xd,s);if(!(e.mode&1))return Ga(n,e,o,null);if(r.data==="$!"){if(i=r.nextSibling&&r.nextSibling.dataset,i)var a=i.dgst;return i=a,s=Error(te(419)),i=_u(s,i,void 0),Ga(n,e,o,i)}if(a=(o&n.childLanes)!==0,ln||a){if(i=Ft,i!==null){switch(o&-o){case 4:r=2;break;case 16:r=8;break;case 64:case 128:case 256:case 512:case 1024:case 2048:case 4096:case 8192:case 16384:case 32768:case 65536:case 131072:case 262144:case 524288:case 1048576:case 2097152:case 4194304:case 8388608:case 16777216:case 33554432:case 67108864:r=32;break;case 536870912:r=268435456;break;default:r=0}r=r&(i.suspendedLanes|o)?0:r,r!==0&&r!==s.retryLane&&(s.retryLane=r,Pi(n,r),Yn(i,n,r,-1))}return Ph(),i=_u(Error(te(421))),Ga(n,e,o,i)}return r.data==="$?"?(e.flags|=128,e.child=n.child,e=RS.bind(null,n),r._reactRetry=e,null):(n=s.treeContext,yn=rr(r.nextSibling),xn=e,ut=!0,Vn=null,n!==null&&(Rn[Cn++]=Mi,Rn[Cn++]=Ei,Rn[Cn++]=Or,Mi=n.id,Ei=n.overflow,Or=e),e=Th(e,i.children),e.flags|=4096,e)}function um(n,e,t){n.lanes|=e;var i=n.alternate;i!==null&&(i.lanes|=e),Bd(n.return,e,t)}function vu(n,e,t,i,r){var s=n.memoizedState;s===null?n.memoizedState={isBackwards:e,rendering:null,renderingStartTime:0,last:i,tail:t,tailMode:r}:(s.isBackwards=e,s.rendering=null,s.renderingStartTime=0,s.last=i,s.tail=t,s.tailMode=r)}function q0(n,e,t){var i=e.pendingProps,r=i.revealOrder,s=i.tail;if(Zt(n,e,i.children,t),i=ht.current,i&2)i=i&1|2,e.flags|=128;else{if(n!==null&&n.flags&128)e:for(n=e.child;n!==null;){if(n.tag===13)n.memoizedState!==null&&um(n,t,e);else if(n.tag===19)um(n,t,e);else if(n.child!==null){n.child.return=n,n=n.child;continue}if(n===e)break e;for(;n.sibling===null;){if(n.return===null||n.return===e)break e;n=n.return}n.sibling.return=n.return,n=n.sibling}i&=1}if(ot(ht,i),!(e.mode&1))e.memoizedState=null;else switch(r){case"forwards":for(t=e.child,r=null;t!==null;)n=t.alternate,n!==null&&ac(n)===null&&(r=t),t=t.sibling;t=r,t===null?(r=e.child,e.child=null):(r=t.sibling,t.sibling=null),vu(e,!1,r,t,s);break;case"backwards":for(t=null,r=e.child,e.child=null;r!==null;){if(n=r.alternate,n!==null&&ac(n)===null){e.child=r;break}n=r.sibling,r.sibling=t,t=r,r=n}vu(e,!0,t,null,s);break;case"together":vu(e,!1,null,null,void 0);break;default:e.memoizedState=null}return e.child}function Dl(n,e){!(e.mode&1)&&n!==null&&(n.alternate=null,e.alternate=null,e.flags|=2)}function Li(n,e,t){if(n!==null&&(e.dependencies=n.dependencies),Br|=e.lanes,!(t&e.childLanes))return null;if(n!==null&&e.child!==n.child)throw Error(te(153));if(e.child!==null){for(n=e.child,t=lr(n,n.pendingProps),e.child=t,t.return=e;n.sibling!==null;)n=n.sibling,t=t.sibling=lr(n,n.pendingProps),t.return=e;t.sibling=null}return e.child}function mS(n,e,t){switch(e.tag){case 3:Y0(e),Hs();break;case 5:S0(e);break;case 1:un(e.type)&&tc(e);break;case 4:gh(e,e.stateNode.containerInfo);break;case 10:var i=e.type._context,r=e.memoizedProps.value;ot(rc,i._currentValue),i._currentValue=r;break;case 13:if(i=e.memoizedState,i!==null)return i.dehydrated!==null?(ot(ht,ht.current&1),e.flags|=128,null):t&e.child.childLanes?K0(n,e,t):(ot(ht,ht.current&1),n=Li(n,e,t),n!==null?n.sibling:null);ot(ht,ht.current&1);break;case 19:if(i=(t&e.childLanes)!==0,n.flags&128){if(i)return q0(n,e,t);e.flags|=128}if(r=e.memoizedState,r!==null&&(r.rendering=null,r.tail=null,r.lastEffect=null),ot(ht,ht.current),i)break;return null;case 22:case 23:return e.lanes=0,X0(n,e,t)}return Li(n,e,t)}var $0,Yd,Z0,Q0;$0=function(n,e){for(var t=e.child;t!==null;){if(t.tag===5||t.tag===6)n.appendChild(t.stateNode);else if(t.tag!==4&&t.child!==null){t.child.return=t,t=t.child;continue}if(t===e)break;for(;t.sibling===null;){if(t.return===null||t.return===e)return;t=t.return}t.sibling.return=t.return,t=t.sibling}};Yd=function(){};Z0=function(n,e,t,i){var r=n.memoizedProps;if(r!==i){n=e.stateNode,Nr(oi.current);var s=null;switch(t){case"input":r=md(n,r),i=md(n,i),s=[];break;case"select":r=mt({},r,{value:void 0}),i=mt({},i,{value:void 0}),s=[];break;case"textarea":r=vd(n,r),i=vd(n,i),s=[];break;default:typeof r.onClick!="function"&&typeof i.onClick=="function"&&(n.onclick=Jl)}xd(t,i);var o;t=null;for(c in r)if(!i.hasOwnProperty(c)&&r.hasOwnProperty(c)&&r[c]!=null)if(c==="style"){var a=r[c];for(o in a)a.hasOwnProperty(o)&&(t||(t={}),t[o]="")}else c!=="dangerouslySetInnerHTML"&&c!=="children"&&c!=="suppressContentEditableWarning"&&c!=="suppressHydrationWarning"&&c!=="autoFocus"&&($o.hasOwnProperty(c)?s||(s=[]):(s=s||[]).push(c,null));for(c in i){var l=i[c];if(a=r!=null?r[c]:void 0,i.hasOwnProperty(c)&&l!==a&&(l!=null||a!=null))if(c==="style")if(a){for(o in a)!a.hasOwnProperty(o)||l&&l.hasOwnProperty(o)||(t||(t={}),t[o]="");for(o in l)l.hasOwnProperty(o)&&a[o]!==l[o]&&(t||(t={}),t[o]=l[o])}else t||(s||(s=[]),s.push(c,t)),t=l;else c==="dangerouslySetInnerHTML"?(l=l?l.__html:void 0,a=a?a.__html:void 0,l!=null&&a!==l&&(s=s||[]).push(c,l)):c==="children"?typeof l!="string"&&typeof l!="number"||(s=s||[]).push(c,""+l):c!=="suppressContentEditableWarning"&&c!=="suppressHydrationWarning"&&($o.hasOwnProperty(c)?(l!=null&&c==="onScroll"&&at("scroll",n),s||a===l||(s=[])):(s=s||[]).push(c,l))}t&&(s=s||[]).push("style",t);var c=s;(e.updateQueue=c)&&(e.flags|=4)}};Q0=function(n,e,t,i){t!==i&&(e.flags|=4)};function _o(n,e){if(!ut)switch(n.tailMode){case"hidden":e=n.tail;for(var t=null;e!==null;)e.alternate!==null&&(t=e),e=e.sibling;t===null?n.tail=null:t.sibling=null;break;case"collapsed":t=n.tail;for(var i=null;t!==null;)t.alternate!==null&&(i=t),t=t.sibling;i===null?e||n.tail===null?n.tail=null:n.tail.sibling=null:i.sibling=null}}function Wt(n){var e=n.alternate!==null&&n.alternate.child===n.child,t=0,i=0;if(e)for(var r=n.child;r!==null;)t|=r.lanes|r.childLanes,i|=r.subtreeFlags&14680064,i|=r.flags&14680064,r.return=n,r=r.sibling;else for(r=n.child;r!==null;)t|=r.lanes|r.childLanes,i|=r.subtreeFlags,i|=r.flags,r.return=n,r=r.sibling;return n.subtreeFlags|=i,n.childLanes=t,e}function gS(n,e,t){var i=e.pendingProps;switch(ch(e),e.tag){case 2:case 16:case 15:case 0:case 11:case 7:case 8:case 12:case 9:case 14:return Wt(e),null;case 1:return un(e.type)&&ec(),Wt(e),null;case 3:return i=e.stateNode,Gs(),ct(cn),ct(Kt),vh(),i.pendingContext&&(i.context=i.pendingContext,i.pendingContext=null),(n===null||n.child===null)&&(Ha(e)?e.flags|=4:n===null||n.memoizedState.isDehydrated&&!(e.flags&256)||(e.flags|=1024,Vn!==null&&(tf(Vn),Vn=null))),Yd(n,e),Wt(e),null;case 5:_h(e);var r=Nr(la.current);if(t=e.type,n!==null&&e.stateNode!=null)Z0(n,e,t,i,r),n.ref!==e.ref&&(e.flags|=512,e.flags|=2097152);else{if(!i){if(e.stateNode===null)throw Error(te(166));return Wt(e),null}if(n=Nr(oi.current),Ha(e)){i=e.stateNode,t=e.type;var s=e.memoizedProps;switch(i[ti]=e,i[oa]=s,n=(e.mode&1)!==0,t){case"dialog":at("cancel",i),at("close",i);break;case"iframe":case"object":case"embed":at("load",i);break;case"video":case"audio":for(r=0;r<Uo.length;r++)at(Uo[r],i);break;case"source":at("error",i);break;case"img":case"image":case"link":at("error",i),at("load",i);break;case"details":at("toggle",i);break;case"input":yp(i,s),at("invalid",i);break;case"select":i._wrapperState={wasMultiple:!!s.multiple},at("invalid",i);break;case"textarea":Sp(i,s),at("invalid",i)}xd(t,s),r=null;for(var o in s)if(s.hasOwnProperty(o)){var a=s[o];o==="children"?typeof a=="string"?i.textContent!==a&&(s.suppressHydrationWarning!==!0&&za(i.textContent,a,n),r=["children",a]):typeof a=="number"&&i.textContent!==""+a&&(s.suppressHydrationWarning!==!0&&za(i.textContent,a,n),r=["children",""+a]):$o.hasOwnProperty(o)&&a!=null&&o==="onScroll"&&at("scroll",i)}switch(t){case"input":Ia(i),xp(i,s,!0);break;case"textarea":Ia(i),Mp(i);break;case"select":case"option":break;default:typeof s.onClick=="function"&&(i.onclick=Jl)}i=r,e.updateQueue=i,i!==null&&(e.flags|=4)}else{o=r.nodeType===9?r:r.ownerDocument,n==="http://www.w3.org/1999/xhtml"&&(n=A_(t)),n==="http://www.w3.org/1999/xhtml"?t==="script"?(n=o.createElement("div"),n.innerHTML="<script><\/script>",n=n.removeChild(n.firstChild)):typeof i.is=="string"?n=o.createElement(t,{is:i.is}):(n=o.createElement(t),t==="select"&&(o=n,i.multiple?o.multiple=!0:i.size&&(o.size=i.size))):n=o.createElementNS(n,t),n[ti]=e,n[oa]=i,$0(n,e,!1,!1),e.stateNode=n;e:{switch(o=Sd(t,i),t){case"dialog":at("cancel",n),at("close",n),r=i;break;case"iframe":case"object":case"embed":at("load",n),r=i;break;case"video":case"audio":for(r=0;r<Uo.length;r++)at(Uo[r],n);r=i;break;case"source":at("error",n),r=i;break;case"img":case"image":case"link":at("error",n),at("load",n),r=i;break;case"details":at("toggle",n),r=i;break;case"input":yp(n,i),r=md(n,i),at("invalid",n);break;case"option":r=i;break;case"select":n._wrapperState={wasMultiple:!!i.multiple},r=mt({},i,{value:void 0}),at("invalid",n);break;case"textarea":Sp(n,i),r=vd(n,i),at("invalid",n);break;default:r=i}xd(t,r),a=r;for(s in a)if(a.hasOwnProperty(s)){var l=a[s];s==="style"?b_(n,l):s==="dangerouslySetInnerHTML"?(l=l?l.__html:void 0,l!=null&&R_(n,l)):s==="children"?typeof l=="string"?(t!=="textarea"||l!=="")&&Zo(n,l):typeof l=="number"&&Zo(n,""+l):s!=="suppressContentEditableWarning"&&s!=="suppressHydrationWarning"&&s!=="autoFocus"&&($o.hasOwnProperty(s)?l!=null&&s==="onScroll"&&at("scroll",n):l!=null&&Kf(n,s,l,o))}switch(t){case"input":Ia(n),xp(n,i,!1);break;case"textarea":Ia(n),Mp(n);break;case"option":i.value!=null&&n.setAttribute("value",""+dr(i.value));break;case"select":n.multiple=!!i.multiple,s=i.value,s!=null?Cs(n,!!i.multiple,s,!1):i.defaultValue!=null&&Cs(n,!!i.multiple,i.defaultValue,!0);break;default:typeof r.onClick=="function"&&(n.onclick=Jl)}switch(t){case"button":case"input":case"select":case"textarea":i=!!i.autoFocus;break e;case"img":i=!0;break e;default:i=!1}}i&&(e.flags|=4)}e.ref!==null&&(e.flags|=512,e.flags|=2097152)}return Wt(e),null;case 6:if(n&&e.stateNode!=null)Q0(n,e,n.memoizedProps,i);else{if(typeof i!="string"&&e.stateNode===null)throw Error(te(166));if(t=Nr(la.current),Nr(oi.current),Ha(e)){if(i=e.stateNode,t=e.memoizedProps,i[ti]=e,(s=i.nodeValue!==t)&&(n=xn,n!==null))switch(n.tag){case 3:za(i.nodeValue,t,(n.mode&1)!==0);break;case 5:n.memoizedProps.suppressHydrationWarning!==!0&&za(i.nodeValue,t,(n.mode&1)!==0)}s&&(e.flags|=4)}else i=(t.nodeType===9?t:t.ownerDocument).createTextNode(i),i[ti]=e,e.stateNode=i}return Wt(e),null;case 13:if(ct(ht),i=e.memoizedState,n===null||n.memoizedState!==null&&n.memoizedState.dehydrated!==null){if(ut&&yn!==null&&e.mode&1&&!(e.flags&128))g0(),Hs(),e.flags|=98560,s=!1;else if(s=Ha(e),i!==null&&i.dehydrated!==null){if(n===null){if(!s)throw Error(te(318));if(s=e.memoizedState,s=s!==null?s.dehydrated:null,!s)throw Error(te(317));s[ti]=e}else Hs(),!(e.flags&128)&&(e.memoizedState=null),e.flags|=4;Wt(e),s=!1}else Vn!==null&&(tf(Vn),Vn=null),s=!0;if(!s)return e.flags&65536?e:null}return e.flags&128?(e.lanes=t,e):(i=i!==null,i!==(n!==null&&n.memoizedState!==null)&&i&&(e.child.flags|=8192,e.mode&1&&(n===null||ht.current&1?bt===0&&(bt=3):Ph())),e.updateQueue!==null&&(e.flags|=4),Wt(e),null);case 4:return Gs(),Yd(n,e),n===null&&ra(e.stateNode.containerInfo),Wt(e),null;case 10:return hh(e.type._context),Wt(e),null;case 17:return un(e.type)&&ec(),Wt(e),null;case 19:if(ct(ht),s=e.memoizedState,s===null)return Wt(e),null;if(i=(e.flags&128)!==0,o=s.rendering,o===null)if(i)_o(s,!1);else{if(bt!==0||n!==null&&n.flags&128)for(n=e.child;n!==null;){if(o=ac(n),o!==null){for(e.flags|=128,_o(s,!1),i=o.updateQueue,i!==null&&(e.updateQueue=i,e.flags|=4),e.subtreeFlags=0,i=t,t=e.child;t!==null;)s=t,n=i,s.flags&=14680066,o=s.alternate,o===null?(s.childLanes=0,s.lanes=n,s.child=null,s.subtreeFlags=0,s.memoizedProps=null,s.memoizedState=null,s.updateQueue=null,s.dependencies=null,s.stateNode=null):(s.childLanes=o.childLanes,s.lanes=o.lanes,s.child=o.child,s.subtreeFlags=0,s.deletions=null,s.memoizedProps=o.memoizedProps,s.memoizedState=o.memoizedState,s.updateQueue=o.updateQueue,s.type=o.type,n=o.dependencies,s.dependencies=n===null?null:{lanes:n.lanes,firstContext:n.firstContext}),t=t.sibling;return ot(ht,ht.current&1|2),e.child}n=n.sibling}s.tail!==null&&xt()>Xs&&(e.flags|=128,i=!0,_o(s,!1),e.lanes=4194304)}else{if(!i)if(n=ac(o),n!==null){if(e.flags|=128,i=!0,t=n.updateQueue,t!==null&&(e.updateQueue=t,e.flags|=4),_o(s,!0),s.tail===null&&s.tailMode==="hidden"&&!o.alternate&&!ut)return Wt(e),null}else 2*xt()-s.renderingStartTime>Xs&&t!==1073741824&&(e.flags|=128,i=!0,_o(s,!1),e.lanes=4194304);s.isBackwards?(o.sibling=e.child,e.child=o):(t=s.last,t!==null?t.sibling=o:e.child=o,s.last=o)}return s.tail!==null?(e=s.tail,s.rendering=e,s.tail=e.sibling,s.renderingStartTime=xt(),e.sibling=null,t=ht.current,ot(ht,i?t&1|2:t&1),e):(Wt(e),null);case 22:case 23:return bh(),i=e.memoizedState!==null,n!==null&&n.memoizedState!==null!==i&&(e.flags|=8192),i&&e.mode&1?vn&1073741824&&(Wt(e),e.subtreeFlags&6&&(e.flags|=8192)):Wt(e),null;case 24:return null;case 25:return null}throw Error(te(156,e.tag))}function _S(n,e){switch(ch(e),e.tag){case 1:return un(e.type)&&ec(),n=e.flags,n&65536?(e.flags=n&-65537|128,e):null;case 3:return Gs(),ct(cn),ct(Kt),vh(),n=e.flags,n&65536&&!(n&128)?(e.flags=n&-65537|128,e):null;case 5:return _h(e),null;case 13:if(ct(ht),n=e.memoizedState,n!==null&&n.dehydrated!==null){if(e.alternate===null)throw Error(te(340));Hs()}return n=e.flags,n&65536?(e.flags=n&-65537|128,e):null;case 19:return ct(ht),null;case 4:return Gs(),null;case 10:return hh(e.type._context),null;case 22:case 23:return bh(),null;case 24:return null;default:return null}}var Wa=!1,Yt=!1,vS=typeof WeakSet=="function"?WeakSet:Set,pe=null;function ws(n,e){var t=n.ref;if(t!==null)if(typeof t=="function")try{t(null)}catch(i){yt(n,e,i)}else t.current=null}function Kd(n,e,t){try{t()}catch(i){yt(n,e,i)}}var dm=!1;function yS(n,e){if(Ld=$l,n=i0(),ah(n)){if("selectionStart"in n)var t={start:n.selectionStart,end:n.selectionEnd};else e:{t=(t=n.ownerDocument)&&t.defaultView||window;var i=t.getSelection&&t.getSelection();if(i&&i.rangeCount!==0){t=i.anchorNode;var r=i.anchorOffset,s=i.focusNode;i=i.focusOffset;try{t.nodeType,s.nodeType}catch{t=null;break e}var o=0,a=-1,l=-1,c=0,u=0,d=n,f=null;t:for(;;){for(var p;d!==t||r!==0&&d.nodeType!==3||(a=o+r),d!==s||i!==0&&d.nodeType!==3||(l=o+i),d.nodeType===3&&(o+=d.nodeValue.length),(p=d.firstChild)!==null;)f=d,d=p;for(;;){if(d===n)break t;if(f===t&&++c===r&&(a=o),f===s&&++u===i&&(l=o),(p=d.nextSibling)!==null)break;d=f,f=d.parentNode}d=p}t=a===-1||l===-1?null:{start:a,end:l}}else t=null}t=t||{start:0,end:0}}else t=null;for(Id={focusedElem:n,selectionRange:t},$l=!1,pe=e;pe!==null;)if(e=pe,n=e.child,(e.subtreeFlags&1028)!==0&&n!==null)n.return=e,pe=n;else for(;pe!==null;){e=pe;try{var g=e.alternate;if(e.flags&1024)switch(e.tag){case 0:case 11:case 15:break;case 1:if(g!==null){var y=g.memoizedProps,m=g.memoizedState,h=e.stateNode,_=h.getSnapshotBeforeUpdate(e.elementType===e.type?y:zn(e.type,y),m);h.__reactInternalSnapshotBeforeUpdate=_}break;case 3:var v=e.stateNode.containerInfo;v.nodeType===1?v.textContent="":v.nodeType===9&&v.documentElement&&v.removeChild(v.documentElement);break;case 5:case 6:case 4:case 17:break;default:throw Error(te(163))}}catch(x){yt(e,e.return,x)}if(n=e.sibling,n!==null){n.return=e.return,pe=n;break}pe=e.return}return g=dm,dm=!1,g}function Wo(n,e,t){var i=e.updateQueue;if(i=i!==null?i.lastEffect:null,i!==null){var r=i=i.next;do{if((r.tag&n)===n){var s=r.destroy;r.destroy=void 0,s!==void 0&&Kd(e,t,s)}r=r.next}while(r!==i)}}function Nc(n,e){if(e=e.updateQueue,e=e!==null?e.lastEffect:null,e!==null){var t=e=e.next;do{if((t.tag&n)===n){var i=t.create;t.destroy=i()}t=t.next}while(t!==e)}}function qd(n){var e=n.ref;if(e!==null){var t=n.stateNode;switch(n.tag){case 5:n=t;break;default:n=t}typeof e=="function"?e(n):e.current=n}}function J0(n){var e=n.alternate;e!==null&&(n.alternate=null,J0(e)),n.child=null,n.deletions=null,n.sibling=null,n.tag===5&&(e=n.stateNode,e!==null&&(delete e[ti],delete e[oa],delete e[Ud],delete e[tS],delete e[nS])),n.stateNode=null,n.return=null,n.dependencies=null,n.memoizedProps=null,n.memoizedState=null,n.pendingProps=null,n.stateNode=null,n.updateQueue=null}function ev(n){return n.tag===5||n.tag===3||n.tag===4}function fm(n){e:for(;;){for(;n.sibling===null;){if(n.return===null||ev(n.return))return null;n=n.return}for(n.sibling.return=n.return,n=n.sibling;n.tag!==5&&n.tag!==6&&n.tag!==18;){if(n.flags&2||n.child===null||n.tag===4)continue e;n.child.return=n,n=n.child}if(!(n.flags&2))return n.stateNode}}function $d(n,e,t){var i=n.tag;if(i===5||i===6)n=n.stateNode,e?t.nodeType===8?t.parentNode.insertBefore(n,e):t.insertBefore(n,e):(t.nodeType===8?(e=t.parentNode,e.insertBefore(n,t)):(e=t,e.appendChild(n)),t=t._reactRootContainer,t!=null||e.onclick!==null||(e.onclick=Jl));else if(i!==4&&(n=n.child,n!==null))for($d(n,e,t),n=n.sibling;n!==null;)$d(n,e,t),n=n.sibling}function Zd(n,e,t){var i=n.tag;if(i===5||i===6)n=n.stateNode,e?t.insertBefore(n,e):t.appendChild(n);else if(i!==4&&(n=n.child,n!==null))for(Zd(n,e,t),n=n.sibling;n!==null;)Zd(n,e,t),n=n.sibling}var Bt=null,Hn=!1;function ki(n,e,t){for(t=t.child;t!==null;)tv(n,e,t),t=t.sibling}function tv(n,e,t){if(si&&typeof si.onCommitFiberUnmount=="function")try{si.onCommitFiberUnmount(wc,t)}catch{}switch(t.tag){case 5:Yt||ws(t,e);case 6:var i=Bt,r=Hn;Bt=null,ki(n,e,t),Bt=i,Hn=r,Bt!==null&&(Hn?(n=Bt,t=t.stateNode,n.nodeType===8?n.parentNode.removeChild(t):n.removeChild(t)):Bt.removeChild(t.stateNode));break;case 18:Bt!==null&&(Hn?(n=Bt,t=t.stateNode,n.nodeType===8?du(n.parentNode,t):n.nodeType===1&&du(n,t),ta(n)):du(Bt,t.stateNode));break;case 4:i=Bt,r=Hn,Bt=t.stateNode.containerInfo,Hn=!0,ki(n,e,t),Bt=i,Hn=r;break;case 0:case 11:case 14:case 15:if(!Yt&&(i=t.updateQueue,i!==null&&(i=i.lastEffect,i!==null))){r=i=i.next;do{var s=r,o=s.destroy;s=s.tag,o!==void 0&&(s&2||s&4)&&Kd(t,e,o),r=r.next}while(r!==i)}ki(n,e,t);break;case 1:if(!Yt&&(ws(t,e),i=t.stateNode,typeof i.componentWillUnmount=="function"))try{i.props=t.memoizedProps,i.state=t.memoizedState,i.componentWillUnmount()}catch(a){yt(t,e,a)}ki(n,e,t);break;case 21:ki(n,e,t);break;case 22:t.mode&1?(Yt=(i=Yt)||t.memoizedState!==null,ki(n,e,t),Yt=i):ki(n,e,t);break;default:ki(n,e,t)}}function hm(n){var e=n.updateQueue;if(e!==null){n.updateQueue=null;var t=n.stateNode;t===null&&(t=n.stateNode=new vS),e.forEach(function(i){var r=CS.bind(null,n,i);t.has(i)||(t.add(i),i.then(r,r))})}}function Fn(n,e){var t=e.deletions;if(t!==null)for(var i=0;i<t.length;i++){var r=t[i];try{var s=n,o=e,a=o;e:for(;a!==null;){switch(a.tag){case 5:Bt=a.stateNode,Hn=!1;break e;case 3:Bt=a.stateNode.containerInfo,Hn=!0;break e;case 4:Bt=a.stateNode.containerInfo,Hn=!0;break e}a=a.return}if(Bt===null)throw Error(te(160));tv(s,o,r),Bt=null,Hn=!1;var l=r.alternate;l!==null&&(l.return=null),r.return=null}catch(c){yt(r,e,c)}}if(e.subtreeFlags&12854)for(e=e.child;e!==null;)nv(e,n),e=e.sibling}function nv(n,e){var t=n.alternate,i=n.flags;switch(n.tag){case 0:case 11:case 14:case 15:if(Fn(e,n),Qn(n),i&4){try{Wo(3,n,n.return),Nc(3,n)}catch(y){yt(n,n.return,y)}try{Wo(5,n,n.return)}catch(y){yt(n,n.return,y)}}break;case 1:Fn(e,n),Qn(n),i&512&&t!==null&&ws(t,t.return);break;case 5:if(Fn(e,n),Qn(n),i&512&&t!==null&&ws(t,t.return),n.flags&32){var r=n.stateNode;try{Zo(r,"")}catch(y){yt(n,n.return,y)}}if(i&4&&(r=n.stateNode,r!=null)){var s=n.memoizedProps,o=t!==null?t.memoizedProps:s,a=n.type,l=n.updateQueue;if(n.updateQueue=null,l!==null)try{a==="input"&&s.type==="radio"&&s.name!=null&&T_(r,s),Sd(a,o);var c=Sd(a,s);for(o=0;o<l.length;o+=2){var u=l[o],d=l[o+1];u==="style"?b_(r,d):u==="dangerouslySetInnerHTML"?R_(r,d):u==="children"?Zo(r,d):Kf(r,u,d,c)}switch(a){case"input":gd(r,s);break;case"textarea":w_(r,s);break;case"select":var f=r._wrapperState.wasMultiple;r._wrapperState.wasMultiple=!!s.multiple;var p=s.value;p!=null?Cs(r,!!s.multiple,p,!1):f!==!!s.multiple&&(s.defaultValue!=null?Cs(r,!!s.multiple,s.defaultValue,!0):Cs(r,!!s.multiple,s.multiple?[]:"",!1))}r[oa]=s}catch(y){yt(n,n.return,y)}}break;case 6:if(Fn(e,n),Qn(n),i&4){if(n.stateNode===null)throw Error(te(162));r=n.stateNode,s=n.memoizedProps;try{r.nodeValue=s}catch(y){yt(n,n.return,y)}}break;case 3:if(Fn(e,n),Qn(n),i&4&&t!==null&&t.memoizedState.isDehydrated)try{ta(e.containerInfo)}catch(y){yt(n,n.return,y)}break;case 4:Fn(e,n),Qn(n);break;case 13:Fn(e,n),Qn(n),r=n.child,r.flags&8192&&(s=r.memoizedState!==null,r.stateNode.isHidden=s,!s||r.alternate!==null&&r.alternate.memoizedState!==null||(Rh=xt())),i&4&&hm(n);break;case 22:if(u=t!==null&&t.memoizedState!==null,n.mode&1?(Yt=(c=Yt)||u,Fn(e,n),Yt=c):Fn(e,n),Qn(n),i&8192){if(c=n.memoizedState!==null,(n.stateNode.isHidden=c)&&!u&&n.mode&1)for(pe=n,u=n.child;u!==null;){for(d=pe=u;pe!==null;){switch(f=pe,p=f.child,f.tag){case 0:case 11:case 14:case 15:Wo(4,f,f.return);break;case 1:ws(f,f.return);var g=f.stateNode;if(typeof g.componentWillUnmount=="function"){i=f,t=f.return;try{e=i,g.props=e.memoizedProps,g.state=e.memoizedState,g.componentWillUnmount()}catch(y){yt(i,t,y)}}break;case 5:ws(f,f.return);break;case 22:if(f.memoizedState!==null){mm(d);continue}}p!==null?(p.return=f,pe=p):mm(d)}u=u.sibling}e:for(u=null,d=n;;){if(d.tag===5){if(u===null){u=d;try{r=d.stateNode,c?(s=r.style,typeof s.setProperty=="function"?s.setProperty("display","none","important"):s.display="none"):(a=d.stateNode,l=d.memoizedProps.style,o=l!=null&&l.hasOwnProperty("display")?l.display:null,a.style.display=C_("display",o))}catch(y){yt(n,n.return,y)}}}else if(d.tag===6){if(u===null)try{d.stateNode.nodeValue=c?"":d.memoizedProps}catch(y){yt(n,n.return,y)}}else if((d.tag!==22&&d.tag!==23||d.memoizedState===null||d===n)&&d.child!==null){d.child.return=d,d=d.child;continue}if(d===n)break e;for(;d.sibling===null;){if(d.return===null||d.return===n)break e;u===d&&(u=null),d=d.return}u===d&&(u=null),d.sibling.return=d.return,d=d.sibling}}break;case 19:Fn(e,n),Qn(n),i&4&&hm(n);break;case 21:break;default:Fn(e,n),Qn(n)}}function Qn(n){var e=n.flags;if(e&2){try{e:{for(var t=n.return;t!==null;){if(ev(t)){var i=t;break e}t=t.return}throw Error(te(160))}switch(i.tag){case 5:var r=i.stateNode;i.flags&32&&(Zo(r,""),i.flags&=-33);var s=fm(n);Zd(n,s,r);break;case 3:case 4:var o=i.stateNode.containerInfo,a=fm(n);$d(n,a,o);break;default:throw Error(te(161))}}catch(l){yt(n,n.return,l)}n.flags&=-3}e&4096&&(n.flags&=-4097)}function xS(n,e,t){pe=n,iv(n)}function iv(n,e,t){for(var i=(n.mode&1)!==0;pe!==null;){var r=pe,s=r.child;if(r.tag===22&&i){var o=r.memoizedState!==null||Wa;if(!o){var a=r.alternate,l=a!==null&&a.memoizedState!==null||Yt;a=Wa;var c=Yt;if(Wa=o,(Yt=l)&&!c)for(pe=r;pe!==null;)o=pe,l=o.child,o.tag===22&&o.memoizedState!==null?gm(r):l!==null?(l.return=o,pe=l):gm(r);for(;s!==null;)pe=s,iv(s),s=s.sibling;pe=r,Wa=a,Yt=c}pm(n)}else r.subtreeFlags&8772&&s!==null?(s.return=r,pe=s):pm(n)}}function pm(n){for(;pe!==null;){var e=pe;if(e.flags&8772){var t=e.alternate;try{if(e.flags&8772)switch(e.tag){case 0:case 11:case 15:Yt||Nc(5,e);break;case 1:var i=e.stateNode;if(e.flags&4&&!Yt)if(t===null)i.componentDidMount();else{var r=e.elementType===e.type?t.memoizedProps:zn(e.type,t.memoizedProps);i.componentDidUpdate(r,t.memoizedState,i.__reactInternalSnapshotBeforeUpdate)}var s=e.updateQueue;s!==null&&Qp(e,s,i);break;case 3:var o=e.updateQueue;if(o!==null){if(t=null,e.child!==null)switch(e.child.tag){case 5:t=e.child.stateNode;break;case 1:t=e.child.stateNode}Qp(e,o,t)}break;case 5:var a=e.stateNode;if(t===null&&e.flags&4){t=a;var l=e.memoizedProps;switch(e.type){case"button":case"input":case"select":case"textarea":l.autoFocus&&t.focus();break;case"img":l.src&&(t.src=l.src)}}break;case 6:break;case 4:break;case 12:break;case 13:if(e.memoizedState===null){var c=e.alternate;if(c!==null){var u=c.memoizedState;if(u!==null){var d=u.dehydrated;d!==null&&ta(d)}}}break;case 19:case 17:case 21:case 22:case 23:case 25:break;default:throw Error(te(163))}Yt||e.flags&512&&qd(e)}catch(f){yt(e,e.return,f)}}if(e===n){pe=null;break}if(t=e.sibling,t!==null){t.return=e.return,pe=t;break}pe=e.return}}function mm(n){for(;pe!==null;){var e=pe;if(e===n){pe=null;break}var t=e.sibling;if(t!==null){t.return=e.return,pe=t;break}pe=e.return}}function gm(n){for(;pe!==null;){var e=pe;try{switch(e.tag){case 0:case 11:case 15:var t=e.return;try{Nc(4,e)}catch(l){yt(e,t,l)}break;case 1:var i=e.stateNode;if(typeof i.componentDidMount=="function"){var r=e.return;try{i.componentDidMount()}catch(l){yt(e,r,l)}}var s=e.return;try{qd(e)}catch(l){yt(e,s,l)}break;case 5:var o=e.return;try{qd(e)}catch(l){yt(e,o,l)}}}catch(l){yt(e,e.return,l)}if(e===n){pe=null;break}var a=e.sibling;if(a!==null){a.return=e.return,pe=a;break}pe=e.return}}var SS=Math.ceil,uc=Ui.ReactCurrentDispatcher,wh=Ui.ReactCurrentOwner,Ln=Ui.ReactCurrentBatchConfig,Ke=0,Ft=null,At=null,zt=0,vn=0,As=pr(0),bt=0,fa=null,Br=0,Dc=0,Ah=0,Xo=null,on=null,Rh=0,Xs=1/0,yi=null,dc=!1,Qd=null,or=null,Xa=!1,Qi=null,fc=0,jo=0,Jd=null,Ul=-1,Fl=0;function nn(){return Ke&6?xt():Ul!==-1?Ul:Ul=xt()}function ar(n){return n.mode&1?Ke&2&&zt!==0?zt&-zt:rS.transition!==null?(Fl===0&&(Fl=H_()),Fl):(n=it,n!==0||(n=window.event,n=n===void 0?16:K_(n.type)),n):1}function Yn(n,e,t,i){if(50<jo)throw jo=0,Jd=null,Error(te(185));ya(n,t,i),(!(Ke&2)||n!==Ft)&&(n===Ft&&(!(Ke&2)&&(Dc|=t),bt===4&&qi(n,zt)),dn(n,i),t===1&&Ke===0&&!(e.mode&1)&&(Xs=xt()+500,Pc&&mr()))}function dn(n,e){var t=n.callbackNode;rx(n,e);var i=ql(n,n===Ft?zt:0);if(i===0)t!==null&&wp(t),n.callbackNode=null,n.callbackPriority=0;else if(e=i&-i,n.callbackPriority!==e){if(t!=null&&wp(t),e===1)n.tag===0?iS(_m.bind(null,n)):h0(_m.bind(null,n)),Jx(function(){!(Ke&6)&&mr()}),t=null;else{switch(V_(i)){case 1:t=Jf;break;case 4:t=B_;break;case 16:t=Kl;break;case 536870912:t=z_;break;default:t=Kl}t=dv(t,rv.bind(null,n))}n.callbackPriority=e,n.callbackNode=t}}function rv(n,e){if(Ul=-1,Fl=0,Ke&6)throw Error(te(327));var t=n.callbackNode;if(Ns()&&n.callbackNode!==t)return null;var i=ql(n,n===Ft?zt:0);if(i===0)return null;if(i&30||i&n.expiredLanes||e)e=hc(n,i);else{e=i;var r=Ke;Ke|=2;var s=ov();(Ft!==n||zt!==e)&&(yi=null,Xs=xt()+500,Dr(n,e));do try{TS();break}catch(a){sv(n,a)}while(!0);fh(),uc.current=s,Ke=r,At!==null?e=0:(Ft=null,zt=0,e=bt)}if(e!==0){if(e===2&&(r=Ad(n),r!==0&&(i=r,e=ef(n,r))),e===1)throw t=fa,Dr(n,0),qi(n,i),dn(n,xt()),t;if(e===6)qi(n,i);else{if(r=n.current.alternate,!(i&30)&&!MS(r)&&(e=hc(n,i),e===2&&(s=Ad(n),s!==0&&(i=s,e=ef(n,s))),e===1))throw t=fa,Dr(n,0),qi(n,i),dn(n,xt()),t;switch(n.finishedWork=r,n.finishedLanes=i,e){case 0:case 1:throw Error(te(345));case 2:Ar(n,on,yi);break;case 3:if(qi(n,i),(i&130023424)===i&&(e=Rh+500-xt(),10<e)){if(ql(n,0)!==0)break;if(r=n.suspendedLanes,(r&i)!==i){nn(),n.pingedLanes|=n.suspendedLanes&r;break}n.timeoutHandle=Dd(Ar.bind(null,n,on,yi),e);break}Ar(n,on,yi);break;case 4:if(qi(n,i),(i&4194240)===i)break;for(e=n.eventTimes,r=-1;0<i;){var o=31-jn(i);s=1<<o,o=e[o],o>r&&(r=o),i&=~s}if(i=r,i=xt()-i,i=(120>i?120:480>i?480:1080>i?1080:1920>i?1920:3e3>i?3e3:4320>i?4320:1960*SS(i/1960))-i,10<i){n.timeoutHandle=Dd(Ar.bind(null,n,on,yi),i);break}Ar(n,on,yi);break;case 5:Ar(n,on,yi);break;default:throw Error(te(329))}}}return dn(n,xt()),n.callbackNode===t?rv.bind(null,n):null}function ef(n,e){var t=Xo;return n.current.memoizedState.isDehydrated&&(Dr(n,e).flags|=256),n=hc(n,e),n!==2&&(e=on,on=t,e!==null&&tf(e)),n}function tf(n){on===null?on=n:on.push.apply(on,n)}function MS(n){for(var e=n;;){if(e.flags&16384){var t=e.updateQueue;if(t!==null&&(t=t.stores,t!==null))for(var i=0;i<t.length;i++){var r=t[i],s=r.getSnapshot;r=r.value;try{if(!$n(s(),r))return!1}catch{return!1}}}if(t=e.child,e.subtreeFlags&16384&&t!==null)t.return=e,e=t;else{if(e===n)break;for(;e.sibling===null;){if(e.return===null||e.return===n)return!0;e=e.return}e.sibling.return=e.return,e=e.sibling}}return!0}function qi(n,e){for(e&=~Ah,e&=~Dc,n.suspendedLanes|=e,n.pingedLanes&=~e,n=n.expirationTimes;0<e;){var t=31-jn(e),i=1<<t;n[t]=-1,e&=~i}}function _m(n){if(Ke&6)throw Error(te(327));Ns();var e=ql(n,0);if(!(e&1))return dn(n,xt()),null;var t=hc(n,e);if(n.tag!==0&&t===2){var i=Ad(n);i!==0&&(e=i,t=ef(n,i))}if(t===1)throw t=fa,Dr(n,0),qi(n,e),dn(n,xt()),t;if(t===6)throw Error(te(345));return n.finishedWork=n.current.alternate,n.finishedLanes=e,Ar(n,on,yi),dn(n,xt()),null}function Ch(n,e){var t=Ke;Ke|=1;try{return n(e)}finally{Ke=t,Ke===0&&(Xs=xt()+500,Pc&&mr())}}function zr(n){Qi!==null&&Qi.tag===0&&!(Ke&6)&&Ns();var e=Ke;Ke|=1;var t=Ln.transition,i=it;try{if(Ln.transition=null,it=1,n)return n()}finally{it=i,Ln.transition=t,Ke=e,!(Ke&6)&&mr()}}function bh(){vn=As.current,ct(As)}function Dr(n,e){n.finishedWork=null,n.finishedLanes=0;var t=n.timeoutHandle;if(t!==-1&&(n.timeoutHandle=-1,Qx(t)),At!==null)for(t=At.return;t!==null;){var i=t;switch(ch(i),i.tag){case 1:i=i.type.childContextTypes,i!=null&&ec();break;case 3:Gs(),ct(cn),ct(Kt),vh();break;case 5:_h(i);break;case 4:Gs();break;case 13:ct(ht);break;case 19:ct(ht);break;case 10:hh(i.type._context);break;case 22:case 23:bh()}t=t.return}if(Ft=n,At=n=lr(n.current,null),zt=vn=e,bt=0,fa=null,Ah=Dc=Br=0,on=Xo=null,Ir!==null){for(e=0;e<Ir.length;e++)if(t=Ir[e],i=t.interleaved,i!==null){t.interleaved=null;var r=i.next,s=t.pending;if(s!==null){var o=s.next;s.next=r,i.next=o}t.pending=i}Ir=null}return n}function sv(n,e){do{var t=At;try{if(fh(),Il.current=cc,lc){for(var i=pt.memoizedState;i!==null;){var r=i.queue;r!==null&&(r.pending=null),i=i.next}lc=!1}if(kr=0,Dt=Ct=pt=null,Go=!1,ca=0,wh.current=null,t===null||t.return===null){bt=1,fa=e,At=null;break}e:{var s=n,o=t.return,a=t,l=e;if(e=zt,a.flags|=32768,l!==null&&typeof l=="object"&&typeof l.then=="function"){var c=l,u=a,d=u.tag;if(!(u.mode&1)&&(d===0||d===11||d===15)){var f=u.alternate;f?(u.updateQueue=f.updateQueue,u.memoizedState=f.memoizedState,u.lanes=f.lanes):(u.updateQueue=null,u.memoizedState=null)}var p=rm(o);if(p!==null){p.flags&=-257,sm(p,o,a,s,e),p.mode&1&&im(s,c,e),e=p,l=c;var g=e.updateQueue;if(g===null){var y=new Set;y.add(l),e.updateQueue=y}else g.add(l);break e}else{if(!(e&1)){im(s,c,e),Ph();break e}l=Error(te(426))}}else if(ut&&a.mode&1){var m=rm(o);if(m!==null){!(m.flags&65536)&&(m.flags|=256),sm(m,o,a,s,e),uh(Ws(l,a));break e}}s=l=Ws(l,a),bt!==4&&(bt=2),Xo===null?Xo=[s]:Xo.push(s),s=o;do{switch(s.tag){case 3:s.flags|=65536,e&=-e,s.lanes|=e;var h=V0(s,l,e);Zp(s,h);break e;case 1:a=l;var _=s.type,v=s.stateNode;if(!(s.flags&128)&&(typeof _.getDerivedStateFromError=="function"||v!==null&&typeof v.componentDidCatch=="function"&&(or===null||!or.has(v)))){s.flags|=65536,e&=-e,s.lanes|=e;var x=G0(s,a,e);Zp(s,x);break e}}s=s.return}while(s!==null)}lv(t)}catch(R){e=R,At===t&&t!==null&&(At=t=t.return);continue}break}while(!0)}function ov(){var n=uc.current;return uc.current=cc,n===null?cc:n}function Ph(){(bt===0||bt===3||bt===2)&&(bt=4),Ft===null||!(Br&268435455)&&!(Dc&268435455)||qi(Ft,zt)}function hc(n,e){var t=Ke;Ke|=2;var i=ov();(Ft!==n||zt!==e)&&(yi=null,Dr(n,e));do try{ES();break}catch(r){sv(n,r)}while(!0);if(fh(),Ke=t,uc.current=i,At!==null)throw Error(te(261));return Ft=null,zt=0,bt}function ES(){for(;At!==null;)av(At)}function TS(){for(;At!==null&&!qy();)av(At)}function av(n){var e=uv(n.alternate,n,vn);n.memoizedProps=n.pendingProps,e===null?lv(n):At=e,wh.current=null}function lv(n){var e=n;do{var t=e.alternate;if(n=e.return,e.flags&32768){if(t=_S(t,e),t!==null){t.flags&=32767,At=t;return}if(n!==null)n.flags|=32768,n.subtreeFlags=0,n.deletions=null;else{bt=6,At=null;return}}else if(t=gS(t,e,vn),t!==null){At=t;return}if(e=e.sibling,e!==null){At=e;return}At=e=n}while(e!==null);bt===0&&(bt=5)}function Ar(n,e,t){var i=it,r=Ln.transition;try{Ln.transition=null,it=1,wS(n,e,t,i)}finally{Ln.transition=r,it=i}return null}function wS(n,e,t,i){do Ns();while(Qi!==null);if(Ke&6)throw Error(te(327));t=n.finishedWork;var r=n.finishedLanes;if(t===null)return null;if(n.finishedWork=null,n.finishedLanes=0,t===n.current)throw Error(te(177));n.callbackNode=null,n.callbackPriority=0;var s=t.lanes|t.childLanes;if(sx(n,s),n===Ft&&(At=Ft=null,zt=0),!(t.subtreeFlags&2064)&&!(t.flags&2064)||Xa||(Xa=!0,dv(Kl,function(){return Ns(),null})),s=(t.flags&15990)!==0,t.subtreeFlags&15990||s){s=Ln.transition,Ln.transition=null;var o=it;it=1;var a=Ke;Ke|=4,wh.current=null,yS(n,t),nv(t,n),Xx(Id),$l=!!Ld,Id=Ld=null,n.current=t,xS(t),$y(),Ke=a,it=o,Ln.transition=s}else n.current=t;if(Xa&&(Xa=!1,Qi=n,fc=r),s=n.pendingLanes,s===0&&(or=null),Jy(t.stateNode),dn(n,xt()),e!==null)for(i=n.onRecoverableError,t=0;t<e.length;t++)r=e[t],i(r.value,{componentStack:r.stack,digest:r.digest});if(dc)throw dc=!1,n=Qd,Qd=null,n;return fc&1&&n.tag!==0&&Ns(),s=n.pendingLanes,s&1?n===Jd?jo++:(jo=0,Jd=n):jo=0,mr(),null}function Ns(){if(Qi!==null){var n=V_(fc),e=Ln.transition,t=it;try{if(Ln.transition=null,it=16>n?16:n,Qi===null)var i=!1;else{if(n=Qi,Qi=null,fc=0,Ke&6)throw Error(te(331));var r=Ke;for(Ke|=4,pe=n.current;pe!==null;){var s=pe,o=s.child;if(pe.flags&16){var a=s.deletions;if(a!==null){for(var l=0;l<a.length;l++){var c=a[l];for(pe=c;pe!==null;){var u=pe;switch(u.tag){case 0:case 11:case 15:Wo(8,u,s)}var d=u.child;if(d!==null)d.return=u,pe=d;else for(;pe!==null;){u=pe;var f=u.sibling,p=u.return;if(J0(u),u===c){pe=null;break}if(f!==null){f.return=p,pe=f;break}pe=p}}}var g=s.alternate;if(g!==null){var y=g.child;if(y!==null){g.child=null;do{var m=y.sibling;y.sibling=null,y=m}while(y!==null)}}pe=s}}if(s.subtreeFlags&2064&&o!==null)o.return=s,pe=o;else e:for(;pe!==null;){if(s=pe,s.flags&2048)switch(s.tag){case 0:case 11:case 15:Wo(9,s,s.return)}var h=s.sibling;if(h!==null){h.return=s.return,pe=h;break e}pe=s.return}}var _=n.current;for(pe=_;pe!==null;){o=pe;var v=o.child;if(o.subtreeFlags&2064&&v!==null)v.return=o,pe=v;else e:for(o=_;pe!==null;){if(a=pe,a.flags&2048)try{switch(a.tag){case 0:case 11:case 15:Nc(9,a)}}catch(R){yt(a,a.return,R)}if(a===o){pe=null;break e}var x=a.sibling;if(x!==null){x.return=a.return,pe=x;break e}pe=a.return}}if(Ke=r,mr(),si&&typeof si.onPostCommitFiberRoot=="function")try{si.onPostCommitFiberRoot(wc,n)}catch{}i=!0}return i}finally{it=t,Ln.transition=e}}return!1}function vm(n,e,t){e=Ws(t,e),e=V0(n,e,1),n=sr(n,e,1),e=nn(),n!==null&&(ya(n,1,e),dn(n,e))}function yt(n,e,t){if(n.tag===3)vm(n,n,t);else for(;e!==null;){if(e.tag===3){vm(e,n,t);break}else if(e.tag===1){var i=e.stateNode;if(typeof e.type.getDerivedStateFromError=="function"||typeof i.componentDidCatch=="function"&&(or===null||!or.has(i))){n=Ws(t,n),n=G0(e,n,1),e=sr(e,n,1),n=nn(),e!==null&&(ya(e,1,n),dn(e,n));break}}e=e.return}}function AS(n,e,t){var i=n.pingCache;i!==null&&i.delete(e),e=nn(),n.pingedLanes|=n.suspendedLanes&t,Ft===n&&(zt&t)===t&&(bt===4||bt===3&&(zt&130023424)===zt&&500>xt()-Rh?Dr(n,0):Ah|=t),dn(n,e)}function cv(n,e){e===0&&(n.mode&1?(e=Ua,Ua<<=1,!(Ua&130023424)&&(Ua=4194304)):e=1);var t=nn();n=Pi(n,e),n!==null&&(ya(n,e,t),dn(n,t))}function RS(n){var e=n.memoizedState,t=0;e!==null&&(t=e.retryLane),cv(n,t)}function CS(n,e){var t=0;switch(n.tag){case 13:var i=n.stateNode,r=n.memoizedState;r!==null&&(t=r.retryLane);break;case 19:i=n.stateNode;break;default:throw Error(te(314))}i!==null&&i.delete(e),cv(n,t)}var uv;uv=function(n,e,t){if(n!==null)if(n.memoizedProps!==e.pendingProps||cn.current)ln=!0;else{if(!(n.lanes&t)&&!(e.flags&128))return ln=!1,mS(n,e,t);ln=!!(n.flags&131072)}else ln=!1,ut&&e.flags&1048576&&p0(e,ic,e.index);switch(e.lanes=0,e.tag){case 2:var i=e.type;Dl(n,e),n=e.pendingProps;var r=zs(e,Kt.current);Is(e,t),r=xh(null,e,i,n,r,t);var s=Sh();return e.flags|=1,typeof r=="object"&&r!==null&&typeof r.render=="function"&&r.$$typeof===void 0?(e.tag=1,e.memoizedState=null,e.updateQueue=null,un(i)?(s=!0,tc(e)):s=!1,e.memoizedState=r.state!==null&&r.state!==void 0?r.state:null,mh(e),r.updater=Ic,e.stateNode=r,r._reactInternals=e,Hd(e,i,n,t),e=Wd(null,e,i,!0,s,t)):(e.tag=0,ut&&s&&lh(e),Zt(null,e,r,t),e=e.child),e;case 16:i=e.elementType;e:{switch(Dl(n,e),n=e.pendingProps,r=i._init,i=r(i._payload),e.type=i,r=e.tag=PS(i),n=zn(i,n),r){case 0:e=Gd(null,e,i,n,t);break e;case 1:e=lm(null,e,i,n,t);break e;case 11:e=om(null,e,i,n,t);break e;case 14:e=am(null,e,i,zn(i.type,n),t);break e}throw Error(te(306,i,""))}return e;case 0:return i=e.type,r=e.pendingProps,r=e.elementType===i?r:zn(i,r),Gd(n,e,i,r,t);case 1:return i=e.type,r=e.pendingProps,r=e.elementType===i?r:zn(i,r),lm(n,e,i,r,t);case 3:e:{if(Y0(e),n===null)throw Error(te(387));i=e.pendingProps,s=e.memoizedState,r=s.element,x0(n,e),oc(e,i,null,t);var o=e.memoizedState;if(i=o.element,s.isDehydrated)if(s={element:i,isDehydrated:!1,cache:o.cache,pendingSuspenseBoundaries:o.pendingSuspenseBoundaries,transitions:o.transitions},e.updateQueue.baseState=s,e.memoizedState=s,e.flags&256){r=Ws(Error(te(423)),e),e=cm(n,e,i,t,r);break e}else if(i!==r){r=Ws(Error(te(424)),e),e=cm(n,e,i,t,r);break e}else for(yn=rr(e.stateNode.containerInfo.firstChild),xn=e,ut=!0,Vn=null,t=v0(e,null,i,t),e.child=t;t;)t.flags=t.flags&-3|4096,t=t.sibling;else{if(Hs(),i===r){e=Li(n,e,t);break e}Zt(n,e,i,t)}e=e.child}return e;case 5:return S0(e),n===null&&kd(e),i=e.type,r=e.pendingProps,s=n!==null?n.memoizedProps:null,o=r.children,Nd(i,r)?o=null:s!==null&&Nd(i,s)&&(e.flags|=32),j0(n,e),Zt(n,e,o,t),e.child;case 6:return n===null&&kd(e),null;case 13:return K0(n,e,t);case 4:return gh(e,e.stateNode.containerInfo),i=e.pendingProps,n===null?e.child=Vs(e,null,i,t):Zt(n,e,i,t),e.child;case 11:return i=e.type,r=e.pendingProps,r=e.elementType===i?r:zn(i,r),om(n,e,i,r,t);case 7:return Zt(n,e,e.pendingProps,t),e.child;case 8:return Zt(n,e,e.pendingProps.children,t),e.child;case 12:return Zt(n,e,e.pendingProps.children,t),e.child;case 10:e:{if(i=e.type._context,r=e.pendingProps,s=e.memoizedProps,o=r.value,ot(rc,i._currentValue),i._currentValue=o,s!==null)if($n(s.value,o)){if(s.children===r.children&&!cn.current){e=Li(n,e,t);break e}}else for(s=e.child,s!==null&&(s.return=e);s!==null;){var a=s.dependencies;if(a!==null){o=s.child;for(var l=a.firstContext;l!==null;){if(l.context===i){if(s.tag===1){l=Ri(-1,t&-t),l.tag=2;var c=s.updateQueue;if(c!==null){c=c.shared;var u=c.pending;u===null?l.next=l:(l.next=u.next,u.next=l),c.pending=l}}s.lanes|=t,l=s.alternate,l!==null&&(l.lanes|=t),Bd(s.return,t,e),a.lanes|=t;break}l=l.next}}else if(s.tag===10)o=s.type===e.type?null:s.child;else if(s.tag===18){if(o=s.return,o===null)throw Error(te(341));o.lanes|=t,a=o.alternate,a!==null&&(a.lanes|=t),Bd(o,t,e),o=s.sibling}else o=s.child;if(o!==null)o.return=s;else for(o=s;o!==null;){if(o===e){o=null;break}if(s=o.sibling,s!==null){s.return=o.return,o=s;break}o=o.return}s=o}Zt(n,e,r.children,t),e=e.child}return e;case 9:return r=e.type,i=e.pendingProps.children,Is(e,t),r=Nn(r),i=i(r),e.flags|=1,Zt(n,e,i,t),e.child;case 14:return i=e.type,r=zn(i,e.pendingProps),r=zn(i.type,r),am(n,e,i,r,t);case 15:return W0(n,e,e.type,e.pendingProps,t);case 17:return i=e.type,r=e.pendingProps,r=e.elementType===i?r:zn(i,r),Dl(n,e),e.tag=1,un(i)?(n=!0,tc(e)):n=!1,Is(e,t),H0(e,i,r),Hd(e,i,r,t),Wd(null,e,i,!0,n,t);case 19:return q0(n,e,t);case 22:return X0(n,e,t)}throw Error(te(156,e.tag))};function dv(n,e){return k_(n,e)}function bS(n,e,t,i){this.tag=n,this.key=t,this.sibling=this.child=this.return=this.stateNode=this.type=this.elementType=null,this.index=0,this.ref=null,this.pendingProps=e,this.dependencies=this.memoizedState=this.updateQueue=this.memoizedProps=null,this.mode=i,this.subtreeFlags=this.flags=0,this.deletions=null,this.childLanes=this.lanes=0,this.alternate=null}function bn(n,e,t,i){return new bS(n,e,t,i)}function Lh(n){return n=n.prototype,!(!n||!n.isReactComponent)}function PS(n){if(typeof n=="function")return Lh(n)?1:0;if(n!=null){if(n=n.$$typeof,n===$f)return 11;if(n===Zf)return 14}return 2}function lr(n,e){var t=n.alternate;return t===null?(t=bn(n.tag,e,n.key,n.mode),t.elementType=n.elementType,t.type=n.type,t.stateNode=n.stateNode,t.alternate=n,n.alternate=t):(t.pendingProps=e,t.type=n.type,t.flags=0,t.subtreeFlags=0,t.deletions=null),t.flags=n.flags&14680064,t.childLanes=n.childLanes,t.lanes=n.lanes,t.child=n.child,t.memoizedProps=n.memoizedProps,t.memoizedState=n.memoizedState,t.updateQueue=n.updateQueue,e=n.dependencies,t.dependencies=e===null?null:{lanes:e.lanes,firstContext:e.firstContext},t.sibling=n.sibling,t.index=n.index,t.ref=n.ref,t}function Ol(n,e,t,i,r,s){var o=2;if(i=n,typeof n=="function")Lh(n)&&(o=1);else if(typeof n=="string")o=5;else e:switch(n){case gs:return Ur(t.children,r,s,e);case qf:o=8,r|=8;break;case dd:return n=bn(12,t,e,r|2),n.elementType=dd,n.lanes=s,n;case fd:return n=bn(13,t,e,r),n.elementType=fd,n.lanes=s,n;case hd:return n=bn(19,t,e,r),n.elementType=hd,n.lanes=s,n;case S_:return Uc(t,r,s,e);default:if(typeof n=="object"&&n!==null)switch(n.$$typeof){case y_:o=10;break e;case x_:o=9;break e;case $f:o=11;break e;case Zf:o=14;break e;case ji:o=16,i=null;break e}throw Error(te(130,n==null?n:typeof n,""))}return e=bn(o,t,e,r),e.elementType=n,e.type=i,e.lanes=s,e}function Ur(n,e,t,i){return n=bn(7,n,i,e),n.lanes=t,n}function Uc(n,e,t,i){return n=bn(22,n,i,e),n.elementType=S_,n.lanes=t,n.stateNode={isHidden:!1},n}function yu(n,e,t){return n=bn(6,n,null,e),n.lanes=t,n}function xu(n,e,t){return e=bn(4,n.children!==null?n.children:[],n.key,e),e.lanes=t,e.stateNode={containerInfo:n.containerInfo,pendingChildren:null,implementation:n.implementation},e}function LS(n,e,t,i,r){this.tag=e,this.containerInfo=n,this.finishedWork=this.pingCache=this.current=this.pendingChildren=null,this.timeoutHandle=-1,this.callbackNode=this.pendingContext=this.context=null,this.callbackPriority=0,this.eventTimes=eu(0),this.expirationTimes=eu(-1),this.entangledLanes=this.finishedLanes=this.mutableReadLanes=this.expiredLanes=this.pingedLanes=this.suspendedLanes=this.pendingLanes=0,this.entanglements=eu(0),this.identifierPrefix=i,this.onRecoverableError=r,this.mutableSourceEagerHydrationData=null}function Ih(n,e,t,i,r,s,o,a,l){return n=new LS(n,e,t,a,l),e===1?(e=1,s===!0&&(e|=8)):e=0,s=bn(3,null,null,e),n.current=s,s.stateNode=n,s.memoizedState={element:i,isDehydrated:t,cache:null,transitions:null,pendingSuspenseBoundaries:null},mh(s),n}function IS(n,e,t){var i=3<arguments.length&&arguments[3]!==void 0?arguments[3]:null;return{$$typeof:ms,key:i==null?null:""+i,children:n,containerInfo:e,implementation:t}}function fv(n){if(!n)return fr;n=n._reactInternals;e:{if(Wr(n)!==n||n.tag!==1)throw Error(te(170));var e=n;do{switch(e.tag){case 3:e=e.stateNode.context;break e;case 1:if(un(e.type)){e=e.stateNode.__reactInternalMemoizedMergedChildContext;break e}}e=e.return}while(e!==null);throw Error(te(171))}if(n.tag===1){var t=n.type;if(un(t))return f0(n,t,e)}return e}function hv(n,e,t,i,r,s,o,a,l){return n=Ih(t,i,!0,n,r,s,o,a,l),n.context=fv(null),t=n.current,i=nn(),r=ar(t),s=Ri(i,r),s.callback=e??null,sr(t,s,r),n.current.lanes=r,ya(n,r,i),dn(n,i),n}function Fc(n,e,t,i){var r=e.current,s=nn(),o=ar(r);return t=fv(t),e.context===null?e.context=t:e.pendingContext=t,e=Ri(s,o),e.payload={element:n},i=i===void 0?null:i,i!==null&&(e.callback=i),n=sr(r,e,o),n!==null&&(Yn(n,r,o,s),Ll(n,r,o)),o}function pc(n){if(n=n.current,!n.child)return null;switch(n.child.tag){case 5:return n.child.stateNode;default:return n.child.stateNode}}function ym(n,e){if(n=n.memoizedState,n!==null&&n.dehydrated!==null){var t=n.retryLane;n.retryLane=t!==0&&t<e?t:e}}function Nh(n,e){ym(n,e),(n=n.alternate)&&ym(n,e)}function NS(){return null}var pv=typeof reportError=="function"?reportError:function(n){console.error(n)};function Dh(n){this._internalRoot=n}Oc.prototype.render=Dh.prototype.render=function(n){var e=this._internalRoot;if(e===null)throw Error(te(409));Fc(n,e,null,null)};Oc.prototype.unmount=Dh.prototype.unmount=function(){var n=this._internalRoot;if(n!==null){this._internalRoot=null;var e=n.containerInfo;zr(function(){Fc(null,n,null,null)}),e[bi]=null}};function Oc(n){this._internalRoot=n}Oc.prototype.unstable_scheduleHydration=function(n){if(n){var e=X_();n={blockedOn:null,target:n,priority:e};for(var t=0;t<Ki.length&&e!==0&&e<Ki[t].priority;t++);Ki.splice(t,0,n),t===0&&Y_(n)}};function Uh(n){return!(!n||n.nodeType!==1&&n.nodeType!==9&&n.nodeType!==11)}function kc(n){return!(!n||n.nodeType!==1&&n.nodeType!==9&&n.nodeType!==11&&(n.nodeType!==8||n.nodeValue!==" react-mount-point-unstable "))}function xm(){}function DS(n,e,t,i,r){if(r){if(typeof i=="function"){var s=i;i=function(){var c=pc(o);s.call(c)}}var o=hv(e,i,n,0,null,!1,!1,"",xm);return n._reactRootContainer=o,n[bi]=o.current,ra(n.nodeType===8?n.parentNode:n),zr(),o}for(;r=n.lastChild;)n.removeChild(r);if(typeof i=="function"){var a=i;i=function(){var c=pc(l);a.call(c)}}var l=Ih(n,0,!1,null,null,!1,!1,"",xm);return n._reactRootContainer=l,n[bi]=l.current,ra(n.nodeType===8?n.parentNode:n),zr(function(){Fc(e,l,t,i)}),l}function Bc(n,e,t,i,r){var s=t._reactRootContainer;if(s){var o=s;if(typeof r=="function"){var a=r;r=function(){var l=pc(o);a.call(l)}}Fc(e,o,n,r)}else o=DS(t,e,n,r,i);return pc(o)}G_=function(n){switch(n.tag){case 3:var e=n.stateNode;if(e.current.memoizedState.isDehydrated){var t=Do(e.pendingLanes);t!==0&&(eh(e,t|1),dn(e,xt()),!(Ke&6)&&(Xs=xt()+500,mr()))}break;case 13:zr(function(){var i=Pi(n,1);if(i!==null){var r=nn();Yn(i,n,1,r)}}),Nh(n,1)}};th=function(n){if(n.tag===13){var e=Pi(n,134217728);if(e!==null){var t=nn();Yn(e,n,134217728,t)}Nh(n,134217728)}};W_=function(n){if(n.tag===13){var e=ar(n),t=Pi(n,e);if(t!==null){var i=nn();Yn(t,n,e,i)}Nh(n,e)}};X_=function(){return it};j_=function(n,e){var t=it;try{return it=n,e()}finally{it=t}};Ed=function(n,e,t){switch(e){case"input":if(gd(n,t),e=t.name,t.type==="radio"&&e!=null){for(t=n;t.parentNode;)t=t.parentNode;for(t=t.querySelectorAll("input[name="+JSON.stringify(""+e)+'][type="radio"]'),e=0;e<t.length;e++){var i=t[e];if(i!==n&&i.form===n.form){var r=bc(i);if(!r)throw Error(te(90));E_(i),gd(i,r)}}}break;case"textarea":w_(n,t);break;case"select":e=t.value,e!=null&&Cs(n,!!t.multiple,e,!1)}};I_=Ch;N_=zr;var US={usingClientEntryPoint:!1,Events:[Sa,xs,bc,P_,L_,Ch]},vo={findFiberByHostInstance:Lr,bundleType:0,version:"18.3.1",rendererPackageName:"react-dom"},FS={bundleType:vo.bundleType,version:vo.version,rendererPackageName:vo.rendererPackageName,rendererConfig:vo.rendererConfig,overrideHookState:null,overrideHookStateDeletePath:null,overrideHookStateRenamePath:null,overrideProps:null,overridePropsDeletePath:null,overridePropsRenamePath:null,setErrorHandler:null,setSuspenseHandler:null,scheduleUpdate:null,currentDispatcherRef:Ui.ReactCurrentDispatcher,findHostInstanceByFiber:function(n){return n=F_(n),n===null?null:n.stateNode},findFiberByHostInstance:vo.findFiberByHostInstance||NS,findHostInstancesForRefresh:null,scheduleRefresh:null,scheduleRoot:null,setRefreshHandler:null,getCurrentFiber:null,reconcilerVersion:"18.3.1-next-f1338f8080-20240426"};if(typeof __REACT_DEVTOOLS_GLOBAL_HOOK__<"u"){var ja=__REACT_DEVTOOLS_GLOBAL_HOOK__;if(!ja.isDisabled&&ja.supportsFiber)try{wc=ja.inject(FS),si=ja}catch{}}Mn.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED=US;Mn.createPortal=function(n,e){var t=2<arguments.length&&arguments[2]!==void 0?arguments[2]:null;if(!Uh(e))throw Error(te(200));return IS(n,e,null,t)};Mn.createRoot=function(n,e){if(!Uh(n))throw Error(te(299));var t=!1,i="",r=pv;return e!=null&&(e.unstable_strictMode===!0&&(t=!0),e.identifierPrefix!==void 0&&(i=e.identifierPrefix),e.onRecoverableError!==void 0&&(r=e.onRecoverableError)),e=Ih(n,1,!1,null,null,t,!1,i,r),n[bi]=e.current,ra(n.nodeType===8?n.parentNode:n),new Dh(e)};Mn.findDOMNode=function(n){if(n==null)return null;if(n.nodeType===1)return n;var e=n._reactInternals;if(e===void 0)throw typeof n.render=="function"?Error(te(188)):(n=Object.keys(n).join(","),Error(te(268,n)));return n=F_(e),n=n===null?null:n.stateNode,n};Mn.flushSync=function(n){return zr(n)};Mn.hydrate=function(n,e,t){if(!kc(e))throw Error(te(200));return Bc(null,n,e,!0,t)};Mn.hydrateRoot=function(n,e,t){if(!Uh(n))throw Error(te(405));var i=t!=null&&t.hydratedSources||null,r=!1,s="",o=pv;if(t!=null&&(t.unstable_strictMode===!0&&(r=!0),t.identifierPrefix!==void 0&&(s=t.identifierPrefix),t.onRecoverableError!==void 0&&(o=t.onRecoverableError)),e=hv(e,null,n,1,t??null,r,!1,s,o),n[bi]=e.current,ra(n),i)for(n=0;n<i.length;n++)t=i[n],r=t._getVersion,r=r(t._source),e.mutableSourceEagerHydrationData==null?e.mutableSourceEagerHydrationData=[t,r]:e.mutableSourceEagerHydrationData.push(t,r);return new Oc(e)};Mn.render=function(n,e,t){if(!kc(e))throw Error(te(200));return Bc(null,n,e,!1,t)};Mn.unmountComponentAtNode=function(n){if(!kc(n))throw Error(te(40));return n._reactRootContainer?(zr(function(){Bc(null,null,n,!1,function(){n._reactRootContainer=null,n[bi]=null})}),!0):!1};Mn.unstable_batchedUpdates=Ch;Mn.unstable_renderSubtreeIntoContainer=function(n,e,t,i){if(!kc(t))throw Error(te(200));if(n==null||n._reactInternals===void 0)throw Error(te(38));return Bc(n,e,t,!1,i)};Mn.version="18.3.1-next-f1338f8080-20240426";function mv(){if(!(typeof __REACT_DEVTOOLS_GLOBAL_HOOK__>"u"||typeof __REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE!="function"))try{__REACT_DEVTOOLS_GLOBAL_HOOK__.checkDCE(mv)}catch(n){console.error(n)}}mv(),m_.exports=Mn;var OS=m_.exports,gv,Sm=OS;gv=Sm.createRoot,Sm.hydrateRoot;function kS(n,e){switch(e.type){case"SET_ATOMS":return{...n,atoms:e.atoms,atomTotal:e.total??e.atoms.length};case"SET_SELECTED":return{...n,selected:e.atom};case"SET_QUERY":return{...n,query:e.query};case"SET_SINCE":return{...n,since:e.since};case"SET_LIVE":return{...n,live:e.live};case"SET_MAX_MODE":return{...n,maxMode:e.maxMode};case"SET_LAYOUT":return{...n,layout:e.layout};case"SET_DAEMON":return{...n,daemonOk:e.ok,daemonVersion:e.version??n.daemonVersion};default:return n}}const BS={atoms:[],selected:null,query:"",since:0,live:!0,maxMode:localStorage.getItem("kurultai-max-mode")==="1",atomTotal:0,layout:"regions",daemonOk:!1,daemonVersion:""},zS=Ne.createContext(null);function Mm(n){let e=2166136261;for(let t=0;t<n.length;t++)e^=n.charCodeAt(t),e=Math.imul(e,16777619)>>>0;return e}function ni(n,e="—"){return n==null||n===""?e:String(n)}function Fh(n){const e=n.atom??n,t=e.metadata??{};return{id:ni(e.id,crypto.randomUUID()),title:ni(e.title,"Untitled memory"),summary:ni(e.summary||e.content||t.summary,"No local summary available."),source:ni(e.source),source_id:ni(e.source_id),tags:Array.isArray(e.tags)?e.tags:Array.isArray(t.tags)?t.tags:[],file:e.file_path||t.file_path||"",indexed_at:e.indexed_at||t.indexed_at||"",last_accessed_at:e.last_accessed_at||t.last_accessed_at||"",score:Number(n.score)||0,tier:e.tier||t.tier||"warm",lean:!1}}function HS(n){const e=n||{};return{id:ni(e.id,crypto.randomUUID()),title:ni(e.title,"Untitled memory"),summary:ni(e.summary,e.tier==="hot"?"No local summary available.":"Lean stub — select to load details."),source:ni(e.source),source_id:ni(e.source_id),tags:Array.isArray(e.tags)?e.tags:[],file:"",indexed_at:e.indexed_at||"",last_accessed_at:e.last_accessed_at||"",score:e.tier==="hot"?1:e.tier==="warm"?.55:.25,tier:e.tier||"warm",lean:!0}}async function Ea(n,e){const t=await fetch(n,{headers:{Accept:"application/json"},signal:e});if(!t.ok)throw new Error(`${n} failed (${t.status})`);return t.json()}async function VS(n){return Ea("/api/status",n)}async function GS(n,e){const t=await Ea(`/api/atoms?limit=${n}`,e);return Array.isArray(t)?t.map(Fh):[]}async function WS(n){const e=await Ea("/api/graph",n);return(e.nodes??[...e.hot??[],...e.warm??[],...e.cold??[]]).map(HS)}async function XS(n){const e=await Ea("/api/activity",n);return Array.isArray(e)?e:[]}async function jS(n,e){const t=await Ea(`/api/search?q=${encodeURIComponent(n)}&limit=20`,e);return Array.isArray(t)?t.map(Fh):[]}async function YS(n,e){const t=await fetch("/api/ask",{method:"POST",headers:{"Content-Type":"application/json",Accept:"application/json"},body:JSON.stringify({question:n}),signal:e});if(!t.ok)throw new Error(`ask failed (${t.status})`);const i=await t.json();return i.answer??i.text??JSON.stringify(i)}async function KS(n){const e=await fetch("/api/touch",{method:"POST",headers:{"Content-Type":"application/json",Accept:"application/json"},body:JSON.stringify({atom_id:n})});if(!e.ok)return null;const t=await e.json();return Fh(t)}async function qS(n){await fetch(`/api/open?file=${encodeURIComponent(n)}`)}function $S(){const n=localStorage.getItem("kurultai-theme");return n==="light"||n==="dark"?n:window.matchMedia("(prefers-color-scheme: light)").matches?"light":"dark"}function ZS({daemonOk:n,daemonVersion:e}){const[t,i]=Ne.useState($S);Ne.useEffect(()=>{document.documentElement.dataset.theme=t,localStorage.setItem("kurultai-theme",t)},[t]);const r=()=>i(s=>s==="dark"?"light":"dark");return H.jsxs("header",{className:"topbar",children:[H.jsxs("a",{className:"brand",href:"/ui/","aria-label":"Kurultai home",children:[H.jsx("span",{className:"brand-mark","aria-hidden":"true",children:H.jsxs("svg",{width:"14",height:"14",viewBox:"0 0 24 24",fill:"none",stroke:"currentColor",strokeWidth:"2.5",strokeLinecap:"round",strokeLinejoin:"round",children:[H.jsx("circle",{cx:"12",cy:"12",r:"3"}),H.jsx("circle",{cx:"19",cy:"5",r:"2"}),H.jsx("circle",{cx:"5",cy:"19",r:"2"}),H.jsx("circle",{cx:"19",cy:"19",r:"2"}),H.jsx("circle",{cx:"5",cy:"5",r:"2"}),H.jsx("line",{x1:"12",y1:"12",x2:"19",y2:"5"}),H.jsx("line",{x1:"12",y1:"12",x2:"5",y2:"19"}),H.jsx("line",{x1:"12",y1:"12",x2:"19",y2:"19"}),H.jsx("line",{x1:"12",y1:"12",x2:"5",y2:"5"})]})}),H.jsx("span",{children:"KURULTAI"}),H.jsx("small",{children:"LOCAL BRAIN"})]}),H.jsxs("nav",{className:"topbar-nav","aria-label":"Site navigation",children:[H.jsx("a",{href:"/ui/index.html",children:"Home"}),H.jsx("a",{href:"/ui/",className:"active",children:"Brain Explorer"})]}),H.jsxs("div",{className:"topbar-status","aria-live":"polite",children:[H.jsx("span",{className:"status-dot",style:{background:n?"var(--electric-dim)":"var(--danger)"}}),H.jsx("span",{id:"daemon-status",children:n?`online${e?" · v"+e:""}`:"connecting"})]}),H.jsxs("button",{id:"theme-toggle",className:"icon-button",type:"button","aria-label":`Switch to ${t==="dark"?"light":"dark"} theme`,"aria-pressed":t==="light",onClick:r,children:[H.jsx("span",{"aria-hidden":"true",children:"◐"}),H.jsx("span",{className:"button-copy",children:"Theme"})]})]})}/**
 * @license
 * Copyright 2010-2024 Three.js Authors
 * SPDX-License-Identifier: MIT
 */const Oh="168",QS=0,Em=1,JS=2,_v=1,eM=2,vi=3,Ii=0,fn=1,ii=2,cr=0,Ds=1,mc=2,Tm=3,wm=4,tM=5,br=100,nM=101,iM=102,rM=103,sM=104,oM=200,aM=201,lM=202,cM=203,nf=204,rf=205,uM=206,dM=207,fM=208,hM=209,pM=210,mM=211,gM=212,_M=213,vM=214,yM=0,xM=1,SM=2,gc=3,MM=4,EM=5,TM=6,wM=7,vv=0,AM=1,RM=2,ur=0,CM=1,bM=2,PM=3,LM=4,IM=5,NM=6,DM=7,Am="attached",UM="detached",yv=300,js=301,Ys=302,sf=303,of=304,zc=306,Ks=1e3,Ji=1001,_c=1002,en=1003,xv=1004,Fo=1005,an=1006,kl=1007,Ti=1008,Ni=1009,Sv=1010,Mv=1011,ha=1012,kh=1013,Hr=1014,Xn=1015,Ta=1016,Bh=1017,zh=1018,qs=1020,Ev=35902,Tv=1021,wv=1022,Pn=1023,Av=1024,Rv=1025,Us=1026,$s=1027,Hh=1028,Vh=1029,Cv=1030,Gh=1031,Wh=1033,Bl=33776,zl=33777,Hl=33778,Vl=33779,af=35840,lf=35841,cf=35842,uf=35843,df=36196,ff=37492,hf=37496,pf=37808,mf=37809,gf=37810,_f=37811,vf=37812,yf=37813,xf=37814,Sf=37815,Mf=37816,Ef=37817,Tf=37818,wf=37819,Af=37820,Rf=37821,Gl=36492,Cf=36494,bf=36495,bv=36283,Pf=36284,Lf=36285,If=36286,pa=2300,ma=2301,Su=2302,Rm=2400,Cm=2401,bm=2402,FM=2500,OM=0,Pv=1,Nf=2,kM=3200,BM=3201,Lv=0,zM=1,$i="",Qt="srgb",Vt="srgb-linear",Xh="display-p3",Hc="display-p3-linear",vc="linear",lt="srgb",yc="rec709",xc="p3",Yr=7680,Pm=519,HM=512,VM=513,GM=514,Iv=515,WM=516,XM=517,jM=518,YM=519,Df=35044,Lm="300 es",wi=2e3,Sc=2001;class so{addEventListener(e,t){this._listeners===void 0&&(this._listeners={});const i=this._listeners;i[e]===void 0&&(i[e]=[]),i[e].indexOf(t)===-1&&i[e].push(t)}hasEventListener(e,t){if(this._listeners===void 0)return!1;const i=this._listeners;return i[e]!==void 0&&i[e].indexOf(t)!==-1}removeEventListener(e,t){if(this._listeners===void 0)return;const r=this._listeners[e];if(r!==void 0){const s=r.indexOf(t);s!==-1&&r.splice(s,1)}}dispatchEvent(e){if(this._listeners===void 0)return;const i=this._listeners[e.type];if(i!==void 0){e.target=this;const r=i.slice(0);for(let s=0,o=r.length;s<o;s++)r[s].call(this,e);e.target=null}}}const Xt=["00","01","02","03","04","05","06","07","08","09","0a","0b","0c","0d","0e","0f","10","11","12","13","14","15","16","17","18","19","1a","1b","1c","1d","1e","1f","20","21","22","23","24","25","26","27","28","29","2a","2b","2c","2d","2e","2f","30","31","32","33","34","35","36","37","38","39","3a","3b","3c","3d","3e","3f","40","41","42","43","44","45","46","47","48","49","4a","4b","4c","4d","4e","4f","50","51","52","53","54","55","56","57","58","59","5a","5b","5c","5d","5e","5f","60","61","62","63","64","65","66","67","68","69","6a","6b","6c","6d","6e","6f","70","71","72","73","74","75","76","77","78","79","7a","7b","7c","7d","7e","7f","80","81","82","83","84","85","86","87","88","89","8a","8b","8c","8d","8e","8f","90","91","92","93","94","95","96","97","98","99","9a","9b","9c","9d","9e","9f","a0","a1","a2","a3","a4","a5","a6","a7","a8","a9","aa","ab","ac","ad","ae","af","b0","b1","b2","b3","b4","b5","b6","b7","b8","b9","ba","bb","bc","bd","be","bf","c0","c1","c2","c3","c4","c5","c6","c7","c8","c9","ca","cb","cc","cd","ce","cf","d0","d1","d2","d3","d4","d5","d6","d7","d8","d9","da","db","dc","dd","de","df","e0","e1","e2","e3","e4","e5","e6","e7","e8","e9","ea","eb","ec","ed","ee","ef","f0","f1","f2","f3","f4","f5","f6","f7","f8","f9","fa","fb","fc","fd","fe","ff"];let Im=1234567;const Yo=Math.PI/180,Zs=180/Math.PI;function Kn(){const n=Math.random()*4294967295|0,e=Math.random()*4294967295|0,t=Math.random()*4294967295|0,i=Math.random()*4294967295|0;return(Xt[n&255]+Xt[n>>8&255]+Xt[n>>16&255]+Xt[n>>24&255]+"-"+Xt[e&255]+Xt[e>>8&255]+"-"+Xt[e>>16&15|64]+Xt[e>>24&255]+"-"+Xt[t&63|128]+Xt[t>>8&255]+"-"+Xt[t>>16&255]+Xt[t>>24&255]+Xt[i&255]+Xt[i>>8&255]+Xt[i>>16&255]+Xt[i>>24&255]).toLowerCase()}function Ut(n,e,t){return Math.max(e,Math.min(t,n))}function jh(n,e){return(n%e+e)%e}function KM(n,e,t,i,r){return i+(n-e)*(r-i)/(t-e)}function qM(n,e,t){return n!==e?(t-n)/(e-n):0}function Ko(n,e,t){return(1-t)*n+t*e}function $M(n,e,t,i){return Ko(n,e,1-Math.exp(-t*i))}function ZM(n,e=1){return e-Math.abs(jh(n,e*2)-e)}function QM(n,e,t){return n<=e?0:n>=t?1:(n=(n-e)/(t-e),n*n*(3-2*n))}function JM(n,e,t){return n<=e?0:n>=t?1:(n=(n-e)/(t-e),n*n*n*(n*(n*6-15)+10))}function eE(n,e){return n+Math.floor(Math.random()*(e-n+1))}function tE(n,e){return n+Math.random()*(e-n)}function nE(n){return n*(.5-Math.random())}function iE(n){n!==void 0&&(Im=n);let e=Im+=1831565813;return e=Math.imul(e^e>>>15,e|1),e^=e+Math.imul(e^e>>>7,e|61),((e^e>>>14)>>>0)/4294967296}function rE(n){return n*Yo}function sE(n){return n*Zs}function oE(n){return(n&n-1)===0&&n!==0}function aE(n){return Math.pow(2,Math.ceil(Math.log(n)/Math.LN2))}function lE(n){return Math.pow(2,Math.floor(Math.log(n)/Math.LN2))}function cE(n,e,t,i,r){const s=Math.cos,o=Math.sin,a=s(t/2),l=o(t/2),c=s((e+i)/2),u=o((e+i)/2),d=s((e-i)/2),f=o((e-i)/2),p=s((i-e)/2),g=o((i-e)/2);switch(r){case"XYX":n.set(a*u,l*d,l*f,a*c);break;case"YZY":n.set(l*f,a*u,l*d,a*c);break;case"ZXZ":n.set(l*d,l*f,a*u,a*c);break;case"XZX":n.set(a*u,l*g,l*p,a*c);break;case"YXY":n.set(l*p,a*u,l*g,a*c);break;case"ZYZ":n.set(l*g,l*p,a*u,a*c);break;default:console.warn("THREE.MathUtils: .setQuaternionFromProperEuler() encountered an unknown order: "+r)}}function Gn(n,e){switch(e.constructor){case Float32Array:return n;case Uint32Array:return n/4294967295;case Uint16Array:return n/65535;case Uint8Array:return n/255;case Int32Array:return Math.max(n/2147483647,-1);case Int16Array:return Math.max(n/32767,-1);case Int8Array:return Math.max(n/127,-1);default:throw new Error("Invalid component type.")}}function tt(n,e){switch(e.constructor){case Float32Array:return n;case Uint32Array:return Math.round(n*4294967295);case Uint16Array:return Math.round(n*65535);case Uint8Array:return Math.round(n*255);case Int32Array:return Math.round(n*2147483647);case Int16Array:return Math.round(n*32767);case Int8Array:return Math.round(n*127);default:throw new Error("Invalid component type.")}}const uE={DEG2RAD:Yo,RAD2DEG:Zs,generateUUID:Kn,clamp:Ut,euclideanModulo:jh,mapLinear:KM,inverseLerp:qM,lerp:Ko,damp:$M,pingpong:ZM,smoothstep:QM,smootherstep:JM,randInt:eE,randFloat:tE,randFloatSpread:nE,seededRandom:iE,degToRad:rE,radToDeg:sE,isPowerOfTwo:oE,ceilPowerOfTwo:aE,floorPowerOfTwo:lE,setQuaternionFromProperEuler:cE,normalize:tt,denormalize:Gn};class we{constructor(e=0,t=0){we.prototype.isVector2=!0,this.x=e,this.y=t}get width(){return this.x}set width(e){this.x=e}get height(){return this.y}set height(e){this.y=e}set(e,t){return this.x=e,this.y=t,this}setScalar(e){return this.x=e,this.y=e,this}setX(e){return this.x=e,this}setY(e){return this.y=e,this}setComponent(e,t){switch(e){case 0:this.x=t;break;case 1:this.y=t;break;default:throw new Error("index is out of range: "+e)}return this}getComponent(e){switch(e){case 0:return this.x;case 1:return this.y;default:throw new Error("index is out of range: "+e)}}clone(){return new this.constructor(this.x,this.y)}copy(e){return this.x=e.x,this.y=e.y,this}add(e){return this.x+=e.x,this.y+=e.y,this}addScalar(e){return this.x+=e,this.y+=e,this}addVectors(e,t){return this.x=e.x+t.x,this.y=e.y+t.y,this}addScaledVector(e,t){return this.x+=e.x*t,this.y+=e.y*t,this}sub(e){return this.x-=e.x,this.y-=e.y,this}subScalar(e){return this.x-=e,this.y-=e,this}subVectors(e,t){return this.x=e.x-t.x,this.y=e.y-t.y,this}multiply(e){return this.x*=e.x,this.y*=e.y,this}multiplyScalar(e){return this.x*=e,this.y*=e,this}divide(e){return this.x/=e.x,this.y/=e.y,this}divideScalar(e){return this.multiplyScalar(1/e)}applyMatrix3(e){const t=this.x,i=this.y,r=e.elements;return this.x=r[0]*t+r[3]*i+r[6],this.y=r[1]*t+r[4]*i+r[7],this}min(e){return this.x=Math.min(this.x,e.x),this.y=Math.min(this.y,e.y),this}max(e){return this.x=Math.max(this.x,e.x),this.y=Math.max(this.y,e.y),this}clamp(e,t){return this.x=Math.max(e.x,Math.min(t.x,this.x)),this.y=Math.max(e.y,Math.min(t.y,this.y)),this}clampScalar(e,t){return this.x=Math.max(e,Math.min(t,this.x)),this.y=Math.max(e,Math.min(t,this.y)),this}clampLength(e,t){const i=this.length();return this.divideScalar(i||1).multiplyScalar(Math.max(e,Math.min(t,i)))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this}negate(){return this.x=-this.x,this.y=-this.y,this}dot(e){return this.x*e.x+this.y*e.y}cross(e){return this.x*e.y-this.y*e.x}lengthSq(){return this.x*this.x+this.y*this.y}length(){return Math.sqrt(this.x*this.x+this.y*this.y)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)}normalize(){return this.divideScalar(this.length()||1)}angle(){return Math.atan2(-this.y,-this.x)+Math.PI}angleTo(e){const t=Math.sqrt(this.lengthSq()*e.lengthSq());if(t===0)return Math.PI/2;const i=this.dot(e)/t;return Math.acos(Ut(i,-1,1))}distanceTo(e){return Math.sqrt(this.distanceToSquared(e))}distanceToSquared(e){const t=this.x-e.x,i=this.y-e.y;return t*t+i*i}manhattanDistanceTo(e){return Math.abs(this.x-e.x)+Math.abs(this.y-e.y)}setLength(e){return this.normalize().multiplyScalar(e)}lerp(e,t){return this.x+=(e.x-this.x)*t,this.y+=(e.y-this.y)*t,this}lerpVectors(e,t,i){return this.x=e.x+(t.x-e.x)*i,this.y=e.y+(t.y-e.y)*i,this}equals(e){return e.x===this.x&&e.y===this.y}fromArray(e,t=0){return this.x=e[t],this.y=e[t+1],this}toArray(e=[],t=0){return e[t]=this.x,e[t+1]=this.y,e}fromBufferAttribute(e,t){return this.x=e.getX(t),this.y=e.getY(t),this}rotateAround(e,t){const i=Math.cos(t),r=Math.sin(t),s=this.x-e.x,o=this.y-e.y;return this.x=s*i-o*r+e.x,this.y=s*r+o*i+e.y,this}random(){return this.x=Math.random(),this.y=Math.random(),this}*[Symbol.iterator](){yield this.x,yield this.y}}class ze{constructor(e,t,i,r,s,o,a,l,c){ze.prototype.isMatrix3=!0,this.elements=[1,0,0,0,1,0,0,0,1],e!==void 0&&this.set(e,t,i,r,s,o,a,l,c)}set(e,t,i,r,s,o,a,l,c){const u=this.elements;return u[0]=e,u[1]=r,u[2]=a,u[3]=t,u[4]=s,u[5]=l,u[6]=i,u[7]=o,u[8]=c,this}identity(){return this.set(1,0,0,0,1,0,0,0,1),this}copy(e){const t=this.elements,i=e.elements;return t[0]=i[0],t[1]=i[1],t[2]=i[2],t[3]=i[3],t[4]=i[4],t[5]=i[5],t[6]=i[6],t[7]=i[7],t[8]=i[8],this}extractBasis(e,t,i){return e.setFromMatrix3Column(this,0),t.setFromMatrix3Column(this,1),i.setFromMatrix3Column(this,2),this}setFromMatrix4(e){const t=e.elements;return this.set(t[0],t[4],t[8],t[1],t[5],t[9],t[2],t[6],t[10]),this}multiply(e){return this.multiplyMatrices(this,e)}premultiply(e){return this.multiplyMatrices(e,this)}multiplyMatrices(e,t){const i=e.elements,r=t.elements,s=this.elements,o=i[0],a=i[3],l=i[6],c=i[1],u=i[4],d=i[7],f=i[2],p=i[5],g=i[8],y=r[0],m=r[3],h=r[6],_=r[1],v=r[4],x=r[7],R=r[2],A=r[5],w=r[8];return s[0]=o*y+a*_+l*R,s[3]=o*m+a*v+l*A,s[6]=o*h+a*x+l*w,s[1]=c*y+u*_+d*R,s[4]=c*m+u*v+d*A,s[7]=c*h+u*x+d*w,s[2]=f*y+p*_+g*R,s[5]=f*m+p*v+g*A,s[8]=f*h+p*x+g*w,this}multiplyScalar(e){const t=this.elements;return t[0]*=e,t[3]*=e,t[6]*=e,t[1]*=e,t[4]*=e,t[7]*=e,t[2]*=e,t[5]*=e,t[8]*=e,this}determinant(){const e=this.elements,t=e[0],i=e[1],r=e[2],s=e[3],o=e[4],a=e[5],l=e[6],c=e[7],u=e[8];return t*o*u-t*a*c-i*s*u+i*a*l+r*s*c-r*o*l}invert(){const e=this.elements,t=e[0],i=e[1],r=e[2],s=e[3],o=e[4],a=e[5],l=e[6],c=e[7],u=e[8],d=u*o-a*c,f=a*l-u*s,p=c*s-o*l,g=t*d+i*f+r*p;if(g===0)return this.set(0,0,0,0,0,0,0,0,0);const y=1/g;return e[0]=d*y,e[1]=(r*c-u*i)*y,e[2]=(a*i-r*o)*y,e[3]=f*y,e[4]=(u*t-r*l)*y,e[5]=(r*s-a*t)*y,e[6]=p*y,e[7]=(i*l-c*t)*y,e[8]=(o*t-i*s)*y,this}transpose(){let e;const t=this.elements;return e=t[1],t[1]=t[3],t[3]=e,e=t[2],t[2]=t[6],t[6]=e,e=t[5],t[5]=t[7],t[7]=e,this}getNormalMatrix(e){return this.setFromMatrix4(e).invert().transpose()}transposeIntoArray(e){const t=this.elements;return e[0]=t[0],e[1]=t[3],e[2]=t[6],e[3]=t[1],e[4]=t[4],e[5]=t[7],e[6]=t[2],e[7]=t[5],e[8]=t[8],this}setUvTransform(e,t,i,r,s,o,a){const l=Math.cos(s),c=Math.sin(s);return this.set(i*l,i*c,-i*(l*o+c*a)+o+e,-r*c,r*l,-r*(-c*o+l*a)+a+t,0,0,1),this}scale(e,t){return this.premultiply(Mu.makeScale(e,t)),this}rotate(e){return this.premultiply(Mu.makeRotation(-e)),this}translate(e,t){return this.premultiply(Mu.makeTranslation(e,t)),this}makeTranslation(e,t){return e.isVector2?this.set(1,0,e.x,0,1,e.y,0,0,1):this.set(1,0,e,0,1,t,0,0,1),this}makeRotation(e){const t=Math.cos(e),i=Math.sin(e);return this.set(t,-i,0,i,t,0,0,0,1),this}makeScale(e,t){return this.set(e,0,0,0,t,0,0,0,1),this}equals(e){const t=this.elements,i=e.elements;for(let r=0;r<9;r++)if(t[r]!==i[r])return!1;return!0}fromArray(e,t=0){for(let i=0;i<9;i++)this.elements[i]=e[i+t];return this}toArray(e=[],t=0){const i=this.elements;return e[t]=i[0],e[t+1]=i[1],e[t+2]=i[2],e[t+3]=i[3],e[t+4]=i[4],e[t+5]=i[5],e[t+6]=i[6],e[t+7]=i[7],e[t+8]=i[8],e}clone(){return new this.constructor().fromArray(this.elements)}}const Mu=new ze;function Nv(n){for(let e=n.length-1;e>=0;--e)if(n[e]>=65535)return!0;return!1}function ga(n){return document.createElementNS("http://www.w3.org/1999/xhtml",n)}function dE(){const n=ga("canvas");return n.style.display="block",n}const Nm={};function Fs(n){n in Nm||(Nm[n]=!0,console.warn(n))}function fE(n,e,t){return new Promise(function(i,r){function s(){switch(n.clientWaitSync(e,n.SYNC_FLUSH_COMMANDS_BIT,0)){case n.WAIT_FAILED:r();break;case n.TIMEOUT_EXPIRED:setTimeout(s,t);break;default:i()}}setTimeout(s,t)})}const Dm=new ze().set(.8224621,.177538,0,.0331941,.9668058,0,.0170827,.0723974,.9105199),Um=new ze().set(1.2249401,-.2249404,0,-.0420569,1.0420571,0,-.0196376,-.0786361,1.0982735),yo={[Vt]:{transfer:vc,primaries:yc,luminanceCoefficients:[.2126,.7152,.0722],toReference:n=>n,fromReference:n=>n},[Qt]:{transfer:lt,primaries:yc,luminanceCoefficients:[.2126,.7152,.0722],toReference:n=>n.convertSRGBToLinear(),fromReference:n=>n.convertLinearToSRGB()},[Hc]:{transfer:vc,primaries:xc,luminanceCoefficients:[.2289,.6917,.0793],toReference:n=>n.applyMatrix3(Um),fromReference:n=>n.applyMatrix3(Dm)},[Xh]:{transfer:lt,primaries:xc,luminanceCoefficients:[.2289,.6917,.0793],toReference:n=>n.convertSRGBToLinear().applyMatrix3(Um),fromReference:n=>n.applyMatrix3(Dm).convertLinearToSRGB()}},hE=new Set([Vt,Hc]),qe={enabled:!0,_workingColorSpace:Vt,get workingColorSpace(){return this._workingColorSpace},set workingColorSpace(n){if(!hE.has(n))throw new Error(`Unsupported working color space, "${n}".`);this._workingColorSpace=n},convert:function(n,e,t){if(this.enabled===!1||e===t||!e||!t)return n;const i=yo[e].toReference,r=yo[t].fromReference;return r(i(n))},fromWorkingColorSpace:function(n,e){return this.convert(n,this._workingColorSpace,e)},toWorkingColorSpace:function(n,e){return this.convert(n,e,this._workingColorSpace)},getPrimaries:function(n){return yo[n].primaries},getTransfer:function(n){return n===$i?vc:yo[n].transfer},getLuminanceCoefficients:function(n,e=this._workingColorSpace){return n.fromArray(yo[e].luminanceCoefficients)}};function Os(n){return n<.04045?n*.0773993808:Math.pow(n*.9478672986+.0521327014,2.4)}function Eu(n){return n<.0031308?n*12.92:1.055*Math.pow(n,.41666)-.055}let Kr;class pE{static getDataURL(e){if(/^data:/i.test(e.src)||typeof HTMLCanvasElement>"u")return e.src;let t;if(e instanceof HTMLCanvasElement)t=e;else{Kr===void 0&&(Kr=ga("canvas")),Kr.width=e.width,Kr.height=e.height;const i=Kr.getContext("2d");e instanceof ImageData?i.putImageData(e,0,0):i.drawImage(e,0,0,e.width,e.height),t=Kr}return t.width>2048||t.height>2048?(console.warn("THREE.ImageUtils.getDataURL: Image converted to jpg for performance reasons",e),t.toDataURL("image/jpeg",.6)):t.toDataURL("image/png")}static sRGBToLinear(e){if(typeof HTMLImageElement<"u"&&e instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&e instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&e instanceof ImageBitmap){const t=ga("canvas");t.width=e.width,t.height=e.height;const i=t.getContext("2d");i.drawImage(e,0,0,e.width,e.height);const r=i.getImageData(0,0,e.width,e.height),s=r.data;for(let o=0;o<s.length;o++)s[o]=Os(s[o]/255)*255;return i.putImageData(r,0,0),t}else if(e.data){const t=e.data.slice(0);for(let i=0;i<t.length;i++)t instanceof Uint8Array||t instanceof Uint8ClampedArray?t[i]=Math.floor(Os(t[i]/255)*255):t[i]=Os(t[i]);return{data:t,width:e.width,height:e.height}}else return console.warn("THREE.ImageUtils.sRGBToLinear(): Unsupported image type. No color space conversion applied."),e}}let mE=0;class Dv{constructor(e=null){this.isSource=!0,Object.defineProperty(this,"id",{value:mE++}),this.uuid=Kn(),this.data=e,this.dataReady=!0,this.version=0}set needsUpdate(e){e===!0&&this.version++}toJSON(e){const t=e===void 0||typeof e=="string";if(!t&&e.images[this.uuid]!==void 0)return e.images[this.uuid];const i={uuid:this.uuid,url:""},r=this.data;if(r!==null){let s;if(Array.isArray(r)){s=[];for(let o=0,a=r.length;o<a;o++)r[o].isDataTexture?s.push(Tu(r[o].image)):s.push(Tu(r[o]))}else s=Tu(r);i.url=s}return t||(e.images[this.uuid]=i),i}}function Tu(n){return typeof HTMLImageElement<"u"&&n instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&n instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&n instanceof ImageBitmap?pE.getDataURL(n):n.data?{data:Array.from(n.data),width:n.width,height:n.height,type:n.data.constructor.name}:(console.warn("THREE.Texture: Unable to serialize Texture."),{})}let gE=0;class Pt extends so{constructor(e=Pt.DEFAULT_IMAGE,t=Pt.DEFAULT_MAPPING,i=Ji,r=Ji,s=an,o=Ti,a=Pn,l=Ni,c=Pt.DEFAULT_ANISOTROPY,u=$i){super(),this.isTexture=!0,Object.defineProperty(this,"id",{value:gE++}),this.uuid=Kn(),this.name="",this.source=new Dv(e),this.mipmaps=[],this.mapping=t,this.channel=0,this.wrapS=i,this.wrapT=r,this.magFilter=s,this.minFilter=o,this.anisotropy=c,this.format=a,this.internalFormat=null,this.type=l,this.offset=new we(0,0),this.repeat=new we(1,1),this.center=new we(0,0),this.rotation=0,this.matrixAutoUpdate=!0,this.matrix=new ze,this.generateMipmaps=!0,this.premultiplyAlpha=!1,this.flipY=!0,this.unpackAlignment=4,this.colorSpace=u,this.userData={},this.version=0,this.onUpdate=null,this.isRenderTargetTexture=!1,this.pmremVersion=0}get image(){return this.source.data}set image(e=null){this.source.data=e}updateMatrix(){this.matrix.setUvTransform(this.offset.x,this.offset.y,this.repeat.x,this.repeat.y,this.rotation,this.center.x,this.center.y)}clone(){return new this.constructor().copy(this)}copy(e){return this.name=e.name,this.source=e.source,this.mipmaps=e.mipmaps.slice(0),this.mapping=e.mapping,this.channel=e.channel,this.wrapS=e.wrapS,this.wrapT=e.wrapT,this.magFilter=e.magFilter,this.minFilter=e.minFilter,this.anisotropy=e.anisotropy,this.format=e.format,this.internalFormat=e.internalFormat,this.type=e.type,this.offset.copy(e.offset),this.repeat.copy(e.repeat),this.center.copy(e.center),this.rotation=e.rotation,this.matrixAutoUpdate=e.matrixAutoUpdate,this.matrix.copy(e.matrix),this.generateMipmaps=e.generateMipmaps,this.premultiplyAlpha=e.premultiplyAlpha,this.flipY=e.flipY,this.unpackAlignment=e.unpackAlignment,this.colorSpace=e.colorSpace,this.userData=JSON.parse(JSON.stringify(e.userData)),this.needsUpdate=!0,this}toJSON(e){const t=e===void 0||typeof e=="string";if(!t&&e.textures[this.uuid]!==void 0)return e.textures[this.uuid];const i={metadata:{version:4.6,type:"Texture",generator:"Texture.toJSON"},uuid:this.uuid,name:this.name,image:this.source.toJSON(e).uuid,mapping:this.mapping,channel:this.channel,repeat:[this.repeat.x,this.repeat.y],offset:[this.offset.x,this.offset.y],center:[this.center.x,this.center.y],rotation:this.rotation,wrap:[this.wrapS,this.wrapT],format:this.format,internalFormat:this.internalFormat,type:this.type,colorSpace:this.colorSpace,minFilter:this.minFilter,magFilter:this.magFilter,anisotropy:this.anisotropy,flipY:this.flipY,generateMipmaps:this.generateMipmaps,premultiplyAlpha:this.premultiplyAlpha,unpackAlignment:this.unpackAlignment};return Object.keys(this.userData).length>0&&(i.userData=this.userData),t||(e.textures[this.uuid]=i),i}dispose(){this.dispatchEvent({type:"dispose"})}transformUv(e){if(this.mapping!==yv)return e;if(e.applyMatrix3(this.matrix),e.x<0||e.x>1)switch(this.wrapS){case Ks:e.x=e.x-Math.floor(e.x);break;case Ji:e.x=e.x<0?0:1;break;case _c:Math.abs(Math.floor(e.x)%2)===1?e.x=Math.ceil(e.x)-e.x:e.x=e.x-Math.floor(e.x);break}if(e.y<0||e.y>1)switch(this.wrapT){case Ks:e.y=e.y-Math.floor(e.y);break;case Ji:e.y=e.y<0?0:1;break;case _c:Math.abs(Math.floor(e.y)%2)===1?e.y=Math.ceil(e.y)-e.y:e.y=e.y-Math.floor(e.y);break}return this.flipY&&(e.y=1-e.y),e}set needsUpdate(e){e===!0&&(this.version++,this.source.needsUpdate=!0)}set needsPMREMUpdate(e){e===!0&&this.pmremVersion++}}Pt.DEFAULT_IMAGE=null;Pt.DEFAULT_MAPPING=yv;Pt.DEFAULT_ANISOTROPY=1;class st{constructor(e=0,t=0,i=0,r=1){st.prototype.isVector4=!0,this.x=e,this.y=t,this.z=i,this.w=r}get width(){return this.z}set width(e){this.z=e}get height(){return this.w}set height(e){this.w=e}set(e,t,i,r){return this.x=e,this.y=t,this.z=i,this.w=r,this}setScalar(e){return this.x=e,this.y=e,this.z=e,this.w=e,this}setX(e){return this.x=e,this}setY(e){return this.y=e,this}setZ(e){return this.z=e,this}setW(e){return this.w=e,this}setComponent(e,t){switch(e){case 0:this.x=t;break;case 1:this.y=t;break;case 2:this.z=t;break;case 3:this.w=t;break;default:throw new Error("index is out of range: "+e)}return this}getComponent(e){switch(e){case 0:return this.x;case 1:return this.y;case 2:return this.z;case 3:return this.w;default:throw new Error("index is out of range: "+e)}}clone(){return new this.constructor(this.x,this.y,this.z,this.w)}copy(e){return this.x=e.x,this.y=e.y,this.z=e.z,this.w=e.w!==void 0?e.w:1,this}add(e){return this.x+=e.x,this.y+=e.y,this.z+=e.z,this.w+=e.w,this}addScalar(e){return this.x+=e,this.y+=e,this.z+=e,this.w+=e,this}addVectors(e,t){return this.x=e.x+t.x,this.y=e.y+t.y,this.z=e.z+t.z,this.w=e.w+t.w,this}addScaledVector(e,t){return this.x+=e.x*t,this.y+=e.y*t,this.z+=e.z*t,this.w+=e.w*t,this}sub(e){return this.x-=e.x,this.y-=e.y,this.z-=e.z,this.w-=e.w,this}subScalar(e){return this.x-=e,this.y-=e,this.z-=e,this.w-=e,this}subVectors(e,t){return this.x=e.x-t.x,this.y=e.y-t.y,this.z=e.z-t.z,this.w=e.w-t.w,this}multiply(e){return this.x*=e.x,this.y*=e.y,this.z*=e.z,this.w*=e.w,this}multiplyScalar(e){return this.x*=e,this.y*=e,this.z*=e,this.w*=e,this}applyMatrix4(e){const t=this.x,i=this.y,r=this.z,s=this.w,o=e.elements;return this.x=o[0]*t+o[4]*i+o[8]*r+o[12]*s,this.y=o[1]*t+o[5]*i+o[9]*r+o[13]*s,this.z=o[2]*t+o[6]*i+o[10]*r+o[14]*s,this.w=o[3]*t+o[7]*i+o[11]*r+o[15]*s,this}divideScalar(e){return this.multiplyScalar(1/e)}setAxisAngleFromQuaternion(e){this.w=2*Math.acos(e.w);const t=Math.sqrt(1-e.w*e.w);return t<1e-4?(this.x=1,this.y=0,this.z=0):(this.x=e.x/t,this.y=e.y/t,this.z=e.z/t),this}setAxisAngleFromRotationMatrix(e){let t,i,r,s;const l=e.elements,c=l[0],u=l[4],d=l[8],f=l[1],p=l[5],g=l[9],y=l[2],m=l[6],h=l[10];if(Math.abs(u-f)<.01&&Math.abs(d-y)<.01&&Math.abs(g-m)<.01){if(Math.abs(u+f)<.1&&Math.abs(d+y)<.1&&Math.abs(g+m)<.1&&Math.abs(c+p+h-3)<.1)return this.set(1,0,0,0),this;t=Math.PI;const v=(c+1)/2,x=(p+1)/2,R=(h+1)/2,A=(u+f)/4,w=(d+y)/4,b=(g+m)/4;return v>x&&v>R?v<.01?(i=0,r=.707106781,s=.707106781):(i=Math.sqrt(v),r=A/i,s=w/i):x>R?x<.01?(i=.707106781,r=0,s=.707106781):(r=Math.sqrt(x),i=A/r,s=b/r):R<.01?(i=.707106781,r=.707106781,s=0):(s=Math.sqrt(R),i=w/s,r=b/s),this.set(i,r,s,t),this}let _=Math.sqrt((m-g)*(m-g)+(d-y)*(d-y)+(f-u)*(f-u));return Math.abs(_)<.001&&(_=1),this.x=(m-g)/_,this.y=(d-y)/_,this.z=(f-u)/_,this.w=Math.acos((c+p+h-1)/2),this}setFromMatrixPosition(e){const t=e.elements;return this.x=t[12],this.y=t[13],this.z=t[14],this.w=t[15],this}min(e){return this.x=Math.min(this.x,e.x),this.y=Math.min(this.y,e.y),this.z=Math.min(this.z,e.z),this.w=Math.min(this.w,e.w),this}max(e){return this.x=Math.max(this.x,e.x),this.y=Math.max(this.y,e.y),this.z=Math.max(this.z,e.z),this.w=Math.max(this.w,e.w),this}clamp(e,t){return this.x=Math.max(e.x,Math.min(t.x,this.x)),this.y=Math.max(e.y,Math.min(t.y,this.y)),this.z=Math.max(e.z,Math.min(t.z,this.z)),this.w=Math.max(e.w,Math.min(t.w,this.w)),this}clampScalar(e,t){return this.x=Math.max(e,Math.min(t,this.x)),this.y=Math.max(e,Math.min(t,this.y)),this.z=Math.max(e,Math.min(t,this.z)),this.w=Math.max(e,Math.min(t,this.w)),this}clampLength(e,t){const i=this.length();return this.divideScalar(i||1).multiplyScalar(Math.max(e,Math.min(t,i)))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this.z=Math.floor(this.z),this.w=Math.floor(this.w),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this.z=Math.ceil(this.z),this.w=Math.ceil(this.w),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this.z=Math.round(this.z),this.w=Math.round(this.w),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this.z=Math.trunc(this.z),this.w=Math.trunc(this.w),this}negate(){return this.x=-this.x,this.y=-this.y,this.z=-this.z,this.w=-this.w,this}dot(e){return this.x*e.x+this.y*e.y+this.z*e.z+this.w*e.w}lengthSq(){return this.x*this.x+this.y*this.y+this.z*this.z+this.w*this.w}length(){return Math.sqrt(this.x*this.x+this.y*this.y+this.z*this.z+this.w*this.w)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)+Math.abs(this.z)+Math.abs(this.w)}normalize(){return this.divideScalar(this.length()||1)}setLength(e){return this.normalize().multiplyScalar(e)}lerp(e,t){return this.x+=(e.x-this.x)*t,this.y+=(e.y-this.y)*t,this.z+=(e.z-this.z)*t,this.w+=(e.w-this.w)*t,this}lerpVectors(e,t,i){return this.x=e.x+(t.x-e.x)*i,this.y=e.y+(t.y-e.y)*i,this.z=e.z+(t.z-e.z)*i,this.w=e.w+(t.w-e.w)*i,this}equals(e){return e.x===this.x&&e.y===this.y&&e.z===this.z&&e.w===this.w}fromArray(e,t=0){return this.x=e[t],this.y=e[t+1],this.z=e[t+2],this.w=e[t+3],this}toArray(e=[],t=0){return e[t]=this.x,e[t+1]=this.y,e[t+2]=this.z,e[t+3]=this.w,e}fromBufferAttribute(e,t){return this.x=e.getX(t),this.y=e.getY(t),this.z=e.getZ(t),this.w=e.getW(t),this}random(){return this.x=Math.random(),this.y=Math.random(),this.z=Math.random(),this.w=Math.random(),this}*[Symbol.iterator](){yield this.x,yield this.y,yield this.z,yield this.w}}class _E extends so{constructor(e=1,t=1,i={}){super(),this.isRenderTarget=!0,this.width=e,this.height=t,this.depth=1,this.scissor=new st(0,0,e,t),this.scissorTest=!1,this.viewport=new st(0,0,e,t);const r={width:e,height:t,depth:1};i=Object.assign({generateMipmaps:!1,internalFormat:null,minFilter:an,depthBuffer:!0,stencilBuffer:!1,resolveDepthBuffer:!0,resolveStencilBuffer:!0,depthTexture:null,samples:0,count:1},i);const s=new Pt(r,i.mapping,i.wrapS,i.wrapT,i.magFilter,i.minFilter,i.format,i.type,i.anisotropy,i.colorSpace);s.flipY=!1,s.generateMipmaps=i.generateMipmaps,s.internalFormat=i.internalFormat,this.textures=[];const o=i.count;for(let a=0;a<o;a++)this.textures[a]=s.clone(),this.textures[a].isRenderTargetTexture=!0;this.depthBuffer=i.depthBuffer,this.stencilBuffer=i.stencilBuffer,this.resolveDepthBuffer=i.resolveDepthBuffer,this.resolveStencilBuffer=i.resolveStencilBuffer,this.depthTexture=i.depthTexture,this.samples=i.samples}get texture(){return this.textures[0]}set texture(e){this.textures[0]=e}setSize(e,t,i=1){if(this.width!==e||this.height!==t||this.depth!==i){this.width=e,this.height=t,this.depth=i;for(let r=0,s=this.textures.length;r<s;r++)this.textures[r].image.width=e,this.textures[r].image.height=t,this.textures[r].image.depth=i;this.dispose()}this.viewport.set(0,0,e,t),this.scissor.set(0,0,e,t)}clone(){return new this.constructor().copy(this)}copy(e){this.width=e.width,this.height=e.height,this.depth=e.depth,this.scissor.copy(e.scissor),this.scissorTest=e.scissorTest,this.viewport.copy(e.viewport),this.textures.length=0;for(let i=0,r=e.textures.length;i<r;i++)this.textures[i]=e.textures[i].clone(),this.textures[i].isRenderTargetTexture=!0;const t=Object.assign({},e.texture.image);return this.texture.source=new Dv(t),this.depthBuffer=e.depthBuffer,this.stencilBuffer=e.stencilBuffer,this.resolveDepthBuffer=e.resolveDepthBuffer,this.resolveStencilBuffer=e.resolveStencilBuffer,e.depthTexture!==null&&(this.depthTexture=e.depthTexture.clone()),this.samples=e.samples,this}dispose(){this.dispatchEvent({type:"dispose"})}}class Vr extends _E{constructor(e=1,t=1,i={}){super(e,t,i),this.isWebGLRenderTarget=!0}}class Uv extends Pt{constructor(e=null,t=1,i=1,r=1){super(null),this.isDataArrayTexture=!0,this.image={data:e,width:t,height:i,depth:r},this.magFilter=en,this.minFilter=en,this.wrapR=Ji,this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1,this.layerUpdates=new Set}addLayerUpdate(e){this.layerUpdates.add(e)}clearLayerUpdates(){this.layerUpdates.clear()}}class vE extends Pt{constructor(e=null,t=1,i=1,r=1){super(null),this.isData3DTexture=!0,this.image={data:e,width:t,height:i,depth:r},this.magFilter=en,this.minFilter=en,this.wrapR=Ji,this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1}}class gr{constructor(e=0,t=0,i=0,r=1){this.isQuaternion=!0,this._x=e,this._y=t,this._z=i,this._w=r}static slerpFlat(e,t,i,r,s,o,a){let l=i[r+0],c=i[r+1],u=i[r+2],d=i[r+3];const f=s[o+0],p=s[o+1],g=s[o+2],y=s[o+3];if(a===0){e[t+0]=l,e[t+1]=c,e[t+2]=u,e[t+3]=d;return}if(a===1){e[t+0]=f,e[t+1]=p,e[t+2]=g,e[t+3]=y;return}if(d!==y||l!==f||c!==p||u!==g){let m=1-a;const h=l*f+c*p+u*g+d*y,_=h>=0?1:-1,v=1-h*h;if(v>Number.EPSILON){const R=Math.sqrt(v),A=Math.atan2(R,h*_);m=Math.sin(m*A)/R,a=Math.sin(a*A)/R}const x=a*_;if(l=l*m+f*x,c=c*m+p*x,u=u*m+g*x,d=d*m+y*x,m===1-a){const R=1/Math.sqrt(l*l+c*c+u*u+d*d);l*=R,c*=R,u*=R,d*=R}}e[t]=l,e[t+1]=c,e[t+2]=u,e[t+3]=d}static multiplyQuaternionsFlat(e,t,i,r,s,o){const a=i[r],l=i[r+1],c=i[r+2],u=i[r+3],d=s[o],f=s[o+1],p=s[o+2],g=s[o+3];return e[t]=a*g+u*d+l*p-c*f,e[t+1]=l*g+u*f+c*d-a*p,e[t+2]=c*g+u*p+a*f-l*d,e[t+3]=u*g-a*d-l*f-c*p,e}get x(){return this._x}set x(e){this._x=e,this._onChangeCallback()}get y(){return this._y}set y(e){this._y=e,this._onChangeCallback()}get z(){return this._z}set z(e){this._z=e,this._onChangeCallback()}get w(){return this._w}set w(e){this._w=e,this._onChangeCallback()}set(e,t,i,r){return this._x=e,this._y=t,this._z=i,this._w=r,this._onChangeCallback(),this}clone(){return new this.constructor(this._x,this._y,this._z,this._w)}copy(e){return this._x=e.x,this._y=e.y,this._z=e.z,this._w=e.w,this._onChangeCallback(),this}setFromEuler(e,t=!0){const i=e._x,r=e._y,s=e._z,o=e._order,a=Math.cos,l=Math.sin,c=a(i/2),u=a(r/2),d=a(s/2),f=l(i/2),p=l(r/2),g=l(s/2);switch(o){case"XYZ":this._x=f*u*d+c*p*g,this._y=c*p*d-f*u*g,this._z=c*u*g+f*p*d,this._w=c*u*d-f*p*g;break;case"YXZ":this._x=f*u*d+c*p*g,this._y=c*p*d-f*u*g,this._z=c*u*g-f*p*d,this._w=c*u*d+f*p*g;break;case"ZXY":this._x=f*u*d-c*p*g,this._y=c*p*d+f*u*g,this._z=c*u*g+f*p*d,this._w=c*u*d-f*p*g;break;case"ZYX":this._x=f*u*d-c*p*g,this._y=c*p*d+f*u*g,this._z=c*u*g-f*p*d,this._w=c*u*d+f*p*g;break;case"YZX":this._x=f*u*d+c*p*g,this._y=c*p*d+f*u*g,this._z=c*u*g-f*p*d,this._w=c*u*d-f*p*g;break;case"XZY":this._x=f*u*d-c*p*g,this._y=c*p*d-f*u*g,this._z=c*u*g+f*p*d,this._w=c*u*d+f*p*g;break;default:console.warn("THREE.Quaternion: .setFromEuler() encountered an unknown order: "+o)}return t===!0&&this._onChangeCallback(),this}setFromAxisAngle(e,t){const i=t/2,r=Math.sin(i);return this._x=e.x*r,this._y=e.y*r,this._z=e.z*r,this._w=Math.cos(i),this._onChangeCallback(),this}setFromRotationMatrix(e){const t=e.elements,i=t[0],r=t[4],s=t[8],o=t[1],a=t[5],l=t[9],c=t[2],u=t[6],d=t[10],f=i+a+d;if(f>0){const p=.5/Math.sqrt(f+1);this._w=.25/p,this._x=(u-l)*p,this._y=(s-c)*p,this._z=(o-r)*p}else if(i>a&&i>d){const p=2*Math.sqrt(1+i-a-d);this._w=(u-l)/p,this._x=.25*p,this._y=(r+o)/p,this._z=(s+c)/p}else if(a>d){const p=2*Math.sqrt(1+a-i-d);this._w=(s-c)/p,this._x=(r+o)/p,this._y=.25*p,this._z=(l+u)/p}else{const p=2*Math.sqrt(1+d-i-a);this._w=(o-r)/p,this._x=(s+c)/p,this._y=(l+u)/p,this._z=.25*p}return this._onChangeCallback(),this}setFromUnitVectors(e,t){let i=e.dot(t)+1;return i<Number.EPSILON?(i=0,Math.abs(e.x)>Math.abs(e.z)?(this._x=-e.y,this._y=e.x,this._z=0,this._w=i):(this._x=0,this._y=-e.z,this._z=e.y,this._w=i)):(this._x=e.y*t.z-e.z*t.y,this._y=e.z*t.x-e.x*t.z,this._z=e.x*t.y-e.y*t.x,this._w=i),this.normalize()}angleTo(e){return 2*Math.acos(Math.abs(Ut(this.dot(e),-1,1)))}rotateTowards(e,t){const i=this.angleTo(e);if(i===0)return this;const r=Math.min(1,t/i);return this.slerp(e,r),this}identity(){return this.set(0,0,0,1)}invert(){return this.conjugate()}conjugate(){return this._x*=-1,this._y*=-1,this._z*=-1,this._onChangeCallback(),this}dot(e){return this._x*e._x+this._y*e._y+this._z*e._z+this._w*e._w}lengthSq(){return this._x*this._x+this._y*this._y+this._z*this._z+this._w*this._w}length(){return Math.sqrt(this._x*this._x+this._y*this._y+this._z*this._z+this._w*this._w)}normalize(){let e=this.length();return e===0?(this._x=0,this._y=0,this._z=0,this._w=1):(e=1/e,this._x=this._x*e,this._y=this._y*e,this._z=this._z*e,this._w=this._w*e),this._onChangeCallback(),this}multiply(e){return this.multiplyQuaternions(this,e)}premultiply(e){return this.multiplyQuaternions(e,this)}multiplyQuaternions(e,t){const i=e._x,r=e._y,s=e._z,o=e._w,a=t._x,l=t._y,c=t._z,u=t._w;return this._x=i*u+o*a+r*c-s*l,this._y=r*u+o*l+s*a-i*c,this._z=s*u+o*c+i*l-r*a,this._w=o*u-i*a-r*l-s*c,this._onChangeCallback(),this}slerp(e,t){if(t===0)return this;if(t===1)return this.copy(e);const i=this._x,r=this._y,s=this._z,o=this._w;let a=o*e._w+i*e._x+r*e._y+s*e._z;if(a<0?(this._w=-e._w,this._x=-e._x,this._y=-e._y,this._z=-e._z,a=-a):this.copy(e),a>=1)return this._w=o,this._x=i,this._y=r,this._z=s,this;const l=1-a*a;if(l<=Number.EPSILON){const p=1-t;return this._w=p*o+t*this._w,this._x=p*i+t*this._x,this._y=p*r+t*this._y,this._z=p*s+t*this._z,this.normalize(),this}const c=Math.sqrt(l),u=Math.atan2(c,a),d=Math.sin((1-t)*u)/c,f=Math.sin(t*u)/c;return this._w=o*d+this._w*f,this._x=i*d+this._x*f,this._y=r*d+this._y*f,this._z=s*d+this._z*f,this._onChangeCallback(),this}slerpQuaternions(e,t,i){return this.copy(e).slerp(t,i)}random(){const e=2*Math.PI*Math.random(),t=2*Math.PI*Math.random(),i=Math.random(),r=Math.sqrt(1-i),s=Math.sqrt(i);return this.set(r*Math.sin(e),r*Math.cos(e),s*Math.sin(t),s*Math.cos(t))}equals(e){return e._x===this._x&&e._y===this._y&&e._z===this._z&&e._w===this._w}fromArray(e,t=0){return this._x=e[t],this._y=e[t+1],this._z=e[t+2],this._w=e[t+3],this._onChangeCallback(),this}toArray(e=[],t=0){return e[t]=this._x,e[t+1]=this._y,e[t+2]=this._z,e[t+3]=this._w,e}fromBufferAttribute(e,t){return this._x=e.getX(t),this._y=e.getY(t),this._z=e.getZ(t),this._w=e.getW(t),this._onChangeCallback(),this}toJSON(){return this.toArray()}_onChange(e){return this._onChangeCallback=e,this}_onChangeCallback(){}*[Symbol.iterator](){yield this._x,yield this._y,yield this._z,yield this._w}}class L{constructor(e=0,t=0,i=0){L.prototype.isVector3=!0,this.x=e,this.y=t,this.z=i}set(e,t,i){return i===void 0&&(i=this.z),this.x=e,this.y=t,this.z=i,this}setScalar(e){return this.x=e,this.y=e,this.z=e,this}setX(e){return this.x=e,this}setY(e){return this.y=e,this}setZ(e){return this.z=e,this}setComponent(e,t){switch(e){case 0:this.x=t;break;case 1:this.y=t;break;case 2:this.z=t;break;default:throw new Error("index is out of range: "+e)}return this}getComponent(e){switch(e){case 0:return this.x;case 1:return this.y;case 2:return this.z;default:throw new Error("index is out of range: "+e)}}clone(){return new this.constructor(this.x,this.y,this.z)}copy(e){return this.x=e.x,this.y=e.y,this.z=e.z,this}add(e){return this.x+=e.x,this.y+=e.y,this.z+=e.z,this}addScalar(e){return this.x+=e,this.y+=e,this.z+=e,this}addVectors(e,t){return this.x=e.x+t.x,this.y=e.y+t.y,this.z=e.z+t.z,this}addScaledVector(e,t){return this.x+=e.x*t,this.y+=e.y*t,this.z+=e.z*t,this}sub(e){return this.x-=e.x,this.y-=e.y,this.z-=e.z,this}subScalar(e){return this.x-=e,this.y-=e,this.z-=e,this}subVectors(e,t){return this.x=e.x-t.x,this.y=e.y-t.y,this.z=e.z-t.z,this}multiply(e){return this.x*=e.x,this.y*=e.y,this.z*=e.z,this}multiplyScalar(e){return this.x*=e,this.y*=e,this.z*=e,this}multiplyVectors(e,t){return this.x=e.x*t.x,this.y=e.y*t.y,this.z=e.z*t.z,this}applyEuler(e){return this.applyQuaternion(Fm.setFromEuler(e))}applyAxisAngle(e,t){return this.applyQuaternion(Fm.setFromAxisAngle(e,t))}applyMatrix3(e){const t=this.x,i=this.y,r=this.z,s=e.elements;return this.x=s[0]*t+s[3]*i+s[6]*r,this.y=s[1]*t+s[4]*i+s[7]*r,this.z=s[2]*t+s[5]*i+s[8]*r,this}applyNormalMatrix(e){return this.applyMatrix3(e).normalize()}applyMatrix4(e){const t=this.x,i=this.y,r=this.z,s=e.elements,o=1/(s[3]*t+s[7]*i+s[11]*r+s[15]);return this.x=(s[0]*t+s[4]*i+s[8]*r+s[12])*o,this.y=(s[1]*t+s[5]*i+s[9]*r+s[13])*o,this.z=(s[2]*t+s[6]*i+s[10]*r+s[14])*o,this}applyQuaternion(e){const t=this.x,i=this.y,r=this.z,s=e.x,o=e.y,a=e.z,l=e.w,c=2*(o*r-a*i),u=2*(a*t-s*r),d=2*(s*i-o*t);return this.x=t+l*c+o*d-a*u,this.y=i+l*u+a*c-s*d,this.z=r+l*d+s*u-o*c,this}project(e){return this.applyMatrix4(e.matrixWorldInverse).applyMatrix4(e.projectionMatrix)}unproject(e){return this.applyMatrix4(e.projectionMatrixInverse).applyMatrix4(e.matrixWorld)}transformDirection(e){const t=this.x,i=this.y,r=this.z,s=e.elements;return this.x=s[0]*t+s[4]*i+s[8]*r,this.y=s[1]*t+s[5]*i+s[9]*r,this.z=s[2]*t+s[6]*i+s[10]*r,this.normalize()}divide(e){return this.x/=e.x,this.y/=e.y,this.z/=e.z,this}divideScalar(e){return this.multiplyScalar(1/e)}min(e){return this.x=Math.min(this.x,e.x),this.y=Math.min(this.y,e.y),this.z=Math.min(this.z,e.z),this}max(e){return this.x=Math.max(this.x,e.x),this.y=Math.max(this.y,e.y),this.z=Math.max(this.z,e.z),this}clamp(e,t){return this.x=Math.max(e.x,Math.min(t.x,this.x)),this.y=Math.max(e.y,Math.min(t.y,this.y)),this.z=Math.max(e.z,Math.min(t.z,this.z)),this}clampScalar(e,t){return this.x=Math.max(e,Math.min(t,this.x)),this.y=Math.max(e,Math.min(t,this.y)),this.z=Math.max(e,Math.min(t,this.z)),this}clampLength(e,t){const i=this.length();return this.divideScalar(i||1).multiplyScalar(Math.max(e,Math.min(t,i)))}floor(){return this.x=Math.floor(this.x),this.y=Math.floor(this.y),this.z=Math.floor(this.z),this}ceil(){return this.x=Math.ceil(this.x),this.y=Math.ceil(this.y),this.z=Math.ceil(this.z),this}round(){return this.x=Math.round(this.x),this.y=Math.round(this.y),this.z=Math.round(this.z),this}roundToZero(){return this.x=Math.trunc(this.x),this.y=Math.trunc(this.y),this.z=Math.trunc(this.z),this}negate(){return this.x=-this.x,this.y=-this.y,this.z=-this.z,this}dot(e){return this.x*e.x+this.y*e.y+this.z*e.z}lengthSq(){return this.x*this.x+this.y*this.y+this.z*this.z}length(){return Math.sqrt(this.x*this.x+this.y*this.y+this.z*this.z)}manhattanLength(){return Math.abs(this.x)+Math.abs(this.y)+Math.abs(this.z)}normalize(){return this.divideScalar(this.length()||1)}setLength(e){return this.normalize().multiplyScalar(e)}lerp(e,t){return this.x+=(e.x-this.x)*t,this.y+=(e.y-this.y)*t,this.z+=(e.z-this.z)*t,this}lerpVectors(e,t,i){return this.x=e.x+(t.x-e.x)*i,this.y=e.y+(t.y-e.y)*i,this.z=e.z+(t.z-e.z)*i,this}cross(e){return this.crossVectors(this,e)}crossVectors(e,t){const i=e.x,r=e.y,s=e.z,o=t.x,a=t.y,l=t.z;return this.x=r*l-s*a,this.y=s*o-i*l,this.z=i*a-r*o,this}projectOnVector(e){const t=e.lengthSq();if(t===0)return this.set(0,0,0);const i=e.dot(this)/t;return this.copy(e).multiplyScalar(i)}projectOnPlane(e){return wu.copy(this).projectOnVector(e),this.sub(wu)}reflect(e){return this.sub(wu.copy(e).multiplyScalar(2*this.dot(e)))}angleTo(e){const t=Math.sqrt(this.lengthSq()*e.lengthSq());if(t===0)return Math.PI/2;const i=this.dot(e)/t;return Math.acos(Ut(i,-1,1))}distanceTo(e){return Math.sqrt(this.distanceToSquared(e))}distanceToSquared(e){const t=this.x-e.x,i=this.y-e.y,r=this.z-e.z;return t*t+i*i+r*r}manhattanDistanceTo(e){return Math.abs(this.x-e.x)+Math.abs(this.y-e.y)+Math.abs(this.z-e.z)}setFromSpherical(e){return this.setFromSphericalCoords(e.radius,e.phi,e.theta)}setFromSphericalCoords(e,t,i){const r=Math.sin(t)*e;return this.x=r*Math.sin(i),this.y=Math.cos(t)*e,this.z=r*Math.cos(i),this}setFromCylindrical(e){return this.setFromCylindricalCoords(e.radius,e.theta,e.y)}setFromCylindricalCoords(e,t,i){return this.x=e*Math.sin(t),this.y=i,this.z=e*Math.cos(t),this}setFromMatrixPosition(e){const t=e.elements;return this.x=t[12],this.y=t[13],this.z=t[14],this}setFromMatrixScale(e){const t=this.setFromMatrixColumn(e,0).length(),i=this.setFromMatrixColumn(e,1).length(),r=this.setFromMatrixColumn(e,2).length();return this.x=t,this.y=i,this.z=r,this}setFromMatrixColumn(e,t){return this.fromArray(e.elements,t*4)}setFromMatrix3Column(e,t){return this.fromArray(e.elements,t*3)}setFromEuler(e){return this.x=e._x,this.y=e._y,this.z=e._z,this}setFromColor(e){return this.x=e.r,this.y=e.g,this.z=e.b,this}equals(e){return e.x===this.x&&e.y===this.y&&e.z===this.z}fromArray(e,t=0){return this.x=e[t],this.y=e[t+1],this.z=e[t+2],this}toArray(e=[],t=0){return e[t]=this.x,e[t+1]=this.y,e[t+2]=this.z,e}fromBufferAttribute(e,t){return this.x=e.getX(t),this.y=e.getY(t),this.z=e.getZ(t),this}random(){return this.x=Math.random(),this.y=Math.random(),this.z=Math.random(),this}randomDirection(){const e=Math.random()*Math.PI*2,t=Math.random()*2-1,i=Math.sqrt(1-t*t);return this.x=i*Math.cos(e),this.y=t,this.z=i*Math.sin(e),this}*[Symbol.iterator](){yield this.x,yield this.y,yield this.z}}const wu=new L,Fm=new gr;class Fi{constructor(e=new L(1/0,1/0,1/0),t=new L(-1/0,-1/0,-1/0)){this.isBox3=!0,this.min=e,this.max=t}set(e,t){return this.min.copy(e),this.max.copy(t),this}setFromArray(e){this.makeEmpty();for(let t=0,i=e.length;t<i;t+=3)this.expandByPoint(On.fromArray(e,t));return this}setFromBufferAttribute(e){this.makeEmpty();for(let t=0,i=e.count;t<i;t++)this.expandByPoint(On.fromBufferAttribute(e,t));return this}setFromPoints(e){this.makeEmpty();for(let t=0,i=e.length;t<i;t++)this.expandByPoint(e[t]);return this}setFromCenterAndSize(e,t){const i=On.copy(t).multiplyScalar(.5);return this.min.copy(e).sub(i),this.max.copy(e).add(i),this}setFromObject(e,t=!1){return this.makeEmpty(),this.expandByObject(e,t)}clone(){return new this.constructor().copy(this)}copy(e){return this.min.copy(e.min),this.max.copy(e.max),this}makeEmpty(){return this.min.x=this.min.y=this.min.z=1/0,this.max.x=this.max.y=this.max.z=-1/0,this}isEmpty(){return this.max.x<this.min.x||this.max.y<this.min.y||this.max.z<this.min.z}getCenter(e){return this.isEmpty()?e.set(0,0,0):e.addVectors(this.min,this.max).multiplyScalar(.5)}getSize(e){return this.isEmpty()?e.set(0,0,0):e.subVectors(this.max,this.min)}expandByPoint(e){return this.min.min(e),this.max.max(e),this}expandByVector(e){return this.min.sub(e),this.max.add(e),this}expandByScalar(e){return this.min.addScalar(-e),this.max.addScalar(e),this}expandByObject(e,t=!1){e.updateWorldMatrix(!1,!1);const i=e.geometry;if(i!==void 0){const s=i.getAttribute("position");if(t===!0&&s!==void 0&&e.isInstancedMesh!==!0)for(let o=0,a=s.count;o<a;o++)e.isMesh===!0?e.getVertexPosition(o,On):On.fromBufferAttribute(s,o),On.applyMatrix4(e.matrixWorld),this.expandByPoint(On);else e.boundingBox!==void 0?(e.boundingBox===null&&e.computeBoundingBox(),Ya.copy(e.boundingBox)):(i.boundingBox===null&&i.computeBoundingBox(),Ya.copy(i.boundingBox)),Ya.applyMatrix4(e.matrixWorld),this.union(Ya)}const r=e.children;for(let s=0,o=r.length;s<o;s++)this.expandByObject(r[s],t);return this}containsPoint(e){return e.x>=this.min.x&&e.x<=this.max.x&&e.y>=this.min.y&&e.y<=this.max.y&&e.z>=this.min.z&&e.z<=this.max.z}containsBox(e){return this.min.x<=e.min.x&&e.max.x<=this.max.x&&this.min.y<=e.min.y&&e.max.y<=this.max.y&&this.min.z<=e.min.z&&e.max.z<=this.max.z}getParameter(e,t){return t.set((e.x-this.min.x)/(this.max.x-this.min.x),(e.y-this.min.y)/(this.max.y-this.min.y),(e.z-this.min.z)/(this.max.z-this.min.z))}intersectsBox(e){return e.max.x>=this.min.x&&e.min.x<=this.max.x&&e.max.y>=this.min.y&&e.min.y<=this.max.y&&e.max.z>=this.min.z&&e.min.z<=this.max.z}intersectsSphere(e){return this.clampPoint(e.center,On),On.distanceToSquared(e.center)<=e.radius*e.radius}intersectsPlane(e){let t,i;return e.normal.x>0?(t=e.normal.x*this.min.x,i=e.normal.x*this.max.x):(t=e.normal.x*this.max.x,i=e.normal.x*this.min.x),e.normal.y>0?(t+=e.normal.y*this.min.y,i+=e.normal.y*this.max.y):(t+=e.normal.y*this.max.y,i+=e.normal.y*this.min.y),e.normal.z>0?(t+=e.normal.z*this.min.z,i+=e.normal.z*this.max.z):(t+=e.normal.z*this.max.z,i+=e.normal.z*this.min.z),t<=-e.constant&&i>=-e.constant}intersectsTriangle(e){if(this.isEmpty())return!1;this.getCenter(xo),Ka.subVectors(this.max,xo),qr.subVectors(e.a,xo),$r.subVectors(e.b,xo),Zr.subVectors(e.c,xo),Bi.subVectors($r,qr),zi.subVectors(Zr,$r),vr.subVectors(qr,Zr);let t=[0,-Bi.z,Bi.y,0,-zi.z,zi.y,0,-vr.z,vr.y,Bi.z,0,-Bi.x,zi.z,0,-zi.x,vr.z,0,-vr.x,-Bi.y,Bi.x,0,-zi.y,zi.x,0,-vr.y,vr.x,0];return!Au(t,qr,$r,Zr,Ka)||(t=[1,0,0,0,1,0,0,0,1],!Au(t,qr,$r,Zr,Ka))?!1:(qa.crossVectors(Bi,zi),t=[qa.x,qa.y,qa.z],Au(t,qr,$r,Zr,Ka))}clampPoint(e,t){return t.copy(e).clamp(this.min,this.max)}distanceToPoint(e){return this.clampPoint(e,On).distanceTo(e)}getBoundingSphere(e){return this.isEmpty()?e.makeEmpty():(this.getCenter(e.center),e.radius=this.getSize(On).length()*.5),e}intersect(e){return this.min.max(e.min),this.max.min(e.max),this.isEmpty()&&this.makeEmpty(),this}union(e){return this.min.min(e.min),this.max.max(e.max),this}applyMatrix4(e){return this.isEmpty()?this:(fi[0].set(this.min.x,this.min.y,this.min.z).applyMatrix4(e),fi[1].set(this.min.x,this.min.y,this.max.z).applyMatrix4(e),fi[2].set(this.min.x,this.max.y,this.min.z).applyMatrix4(e),fi[3].set(this.min.x,this.max.y,this.max.z).applyMatrix4(e),fi[4].set(this.max.x,this.min.y,this.min.z).applyMatrix4(e),fi[5].set(this.max.x,this.min.y,this.max.z).applyMatrix4(e),fi[6].set(this.max.x,this.max.y,this.min.z).applyMatrix4(e),fi[7].set(this.max.x,this.max.y,this.max.z).applyMatrix4(e),this.setFromPoints(fi),this)}translate(e){return this.min.add(e),this.max.add(e),this}equals(e){return e.min.equals(this.min)&&e.max.equals(this.max)}}const fi=[new L,new L,new L,new L,new L,new L,new L,new L],On=new L,Ya=new Fi,qr=new L,$r=new L,Zr=new L,Bi=new L,zi=new L,vr=new L,xo=new L,Ka=new L,qa=new L,yr=new L;function Au(n,e,t,i,r){for(let s=0,o=n.length-3;s<=o;s+=3){yr.fromArray(n,s);const a=r.x*Math.abs(yr.x)+r.y*Math.abs(yr.y)+r.z*Math.abs(yr.z),l=e.dot(yr),c=t.dot(yr),u=i.dot(yr);if(Math.max(-Math.max(l,c,u),Math.min(l,c,u))>a)return!1}return!0}const yE=new Fi,So=new L,Ru=new L;class li{constructor(e=new L,t=-1){this.isSphere=!0,this.center=e,this.radius=t}set(e,t){return this.center.copy(e),this.radius=t,this}setFromPoints(e,t){const i=this.center;t!==void 0?i.copy(t):yE.setFromPoints(e).getCenter(i);let r=0;for(let s=0,o=e.length;s<o;s++)r=Math.max(r,i.distanceToSquared(e[s]));return this.radius=Math.sqrt(r),this}copy(e){return this.center.copy(e.center),this.radius=e.radius,this}isEmpty(){return this.radius<0}makeEmpty(){return this.center.set(0,0,0),this.radius=-1,this}containsPoint(e){return e.distanceToSquared(this.center)<=this.radius*this.radius}distanceToPoint(e){return e.distanceTo(this.center)-this.radius}intersectsSphere(e){const t=this.radius+e.radius;return e.center.distanceToSquared(this.center)<=t*t}intersectsBox(e){return e.intersectsSphere(this)}intersectsPlane(e){return Math.abs(e.distanceToPoint(this.center))<=this.radius}clampPoint(e,t){const i=this.center.distanceToSquared(e);return t.copy(e),i>this.radius*this.radius&&(t.sub(this.center).normalize(),t.multiplyScalar(this.radius).add(this.center)),t}getBoundingBox(e){return this.isEmpty()?(e.makeEmpty(),e):(e.set(this.center,this.center),e.expandByScalar(this.radius),e)}applyMatrix4(e){return this.center.applyMatrix4(e),this.radius=this.radius*e.getMaxScaleOnAxis(),this}translate(e){return this.center.add(e),this}expandByPoint(e){if(this.isEmpty())return this.center.copy(e),this.radius=0,this;So.subVectors(e,this.center);const t=So.lengthSq();if(t>this.radius*this.radius){const i=Math.sqrt(t),r=(i-this.radius)*.5;this.center.addScaledVector(So,r/i),this.radius+=r}return this}union(e){return e.isEmpty()?this:this.isEmpty()?(this.copy(e),this):(this.center.equals(e.center)===!0?this.radius=Math.max(this.radius,e.radius):(Ru.subVectors(e.center,this.center).setLength(e.radius),this.expandByPoint(So.copy(e.center).add(Ru)),this.expandByPoint(So.copy(e.center).sub(Ru))),this)}equals(e){return e.center.equals(this.center)&&e.radius===this.radius}clone(){return new this.constructor().copy(this)}}const hi=new L,Cu=new L,$a=new L,Hi=new L,bu=new L,Za=new L,Pu=new L;class wa{constructor(e=new L,t=new L(0,0,-1)){this.origin=e,this.direction=t}set(e,t){return this.origin.copy(e),this.direction.copy(t),this}copy(e){return this.origin.copy(e.origin),this.direction.copy(e.direction),this}at(e,t){return t.copy(this.origin).addScaledVector(this.direction,e)}lookAt(e){return this.direction.copy(e).sub(this.origin).normalize(),this}recast(e){return this.origin.copy(this.at(e,hi)),this}closestPointToPoint(e,t){t.subVectors(e,this.origin);const i=t.dot(this.direction);return i<0?t.copy(this.origin):t.copy(this.origin).addScaledVector(this.direction,i)}distanceToPoint(e){return Math.sqrt(this.distanceSqToPoint(e))}distanceSqToPoint(e){const t=hi.subVectors(e,this.origin).dot(this.direction);return t<0?this.origin.distanceToSquared(e):(hi.copy(this.origin).addScaledVector(this.direction,t),hi.distanceToSquared(e))}distanceSqToSegment(e,t,i,r){Cu.copy(e).add(t).multiplyScalar(.5),$a.copy(t).sub(e).normalize(),Hi.copy(this.origin).sub(Cu);const s=e.distanceTo(t)*.5,o=-this.direction.dot($a),a=Hi.dot(this.direction),l=-Hi.dot($a),c=Hi.lengthSq(),u=Math.abs(1-o*o);let d,f,p,g;if(u>0)if(d=o*l-a,f=o*a-l,g=s*u,d>=0)if(f>=-g)if(f<=g){const y=1/u;d*=y,f*=y,p=d*(d+o*f+2*a)+f*(o*d+f+2*l)+c}else f=s,d=Math.max(0,-(o*f+a)),p=-d*d+f*(f+2*l)+c;else f=-s,d=Math.max(0,-(o*f+a)),p=-d*d+f*(f+2*l)+c;else f<=-g?(d=Math.max(0,-(-o*s+a)),f=d>0?-s:Math.min(Math.max(-s,-l),s),p=-d*d+f*(f+2*l)+c):f<=g?(d=0,f=Math.min(Math.max(-s,-l),s),p=f*(f+2*l)+c):(d=Math.max(0,-(o*s+a)),f=d>0?s:Math.min(Math.max(-s,-l),s),p=-d*d+f*(f+2*l)+c);else f=o>0?-s:s,d=Math.max(0,-(o*f+a)),p=-d*d+f*(f+2*l)+c;return i&&i.copy(this.origin).addScaledVector(this.direction,d),r&&r.copy(Cu).addScaledVector($a,f),p}intersectSphere(e,t){hi.subVectors(e.center,this.origin);const i=hi.dot(this.direction),r=hi.dot(hi)-i*i,s=e.radius*e.radius;if(r>s)return null;const o=Math.sqrt(s-r),a=i-o,l=i+o;return l<0?null:a<0?this.at(l,t):this.at(a,t)}intersectsSphere(e){return this.distanceSqToPoint(e.center)<=e.radius*e.radius}distanceToPlane(e){const t=e.normal.dot(this.direction);if(t===0)return e.distanceToPoint(this.origin)===0?0:null;const i=-(this.origin.dot(e.normal)+e.constant)/t;return i>=0?i:null}intersectPlane(e,t){const i=this.distanceToPlane(e);return i===null?null:this.at(i,t)}intersectsPlane(e){const t=e.distanceToPoint(this.origin);return t===0||e.normal.dot(this.direction)*t<0}intersectBox(e,t){let i,r,s,o,a,l;const c=1/this.direction.x,u=1/this.direction.y,d=1/this.direction.z,f=this.origin;return c>=0?(i=(e.min.x-f.x)*c,r=(e.max.x-f.x)*c):(i=(e.max.x-f.x)*c,r=(e.min.x-f.x)*c),u>=0?(s=(e.min.y-f.y)*u,o=(e.max.y-f.y)*u):(s=(e.max.y-f.y)*u,o=(e.min.y-f.y)*u),i>o||s>r||((s>i||isNaN(i))&&(i=s),(o<r||isNaN(r))&&(r=o),d>=0?(a=(e.min.z-f.z)*d,l=(e.max.z-f.z)*d):(a=(e.max.z-f.z)*d,l=(e.min.z-f.z)*d),i>l||a>r)||((a>i||i!==i)&&(i=a),(l<r||r!==r)&&(r=l),r<0)?null:this.at(i>=0?i:r,t)}intersectsBox(e){return this.intersectBox(e,hi)!==null}intersectTriangle(e,t,i,r,s){bu.subVectors(t,e),Za.subVectors(i,e),Pu.crossVectors(bu,Za);let o=this.direction.dot(Pu),a;if(o>0){if(r)return null;a=1}else if(o<0)a=-1,o=-o;else return null;Hi.subVectors(this.origin,e);const l=a*this.direction.dot(Za.crossVectors(Hi,Za));if(l<0)return null;const c=a*this.direction.dot(bu.cross(Hi));if(c<0||l+c>o)return null;const u=-a*Hi.dot(Pu);return u<0?null:this.at(u/o,s)}applyMatrix4(e){return this.origin.applyMatrix4(e),this.direction.transformDirection(e),this}equals(e){return e.origin.equals(this.origin)&&e.direction.equals(this.direction)}clone(){return new this.constructor().copy(this)}}class Ue{constructor(e,t,i,r,s,o,a,l,c,u,d,f,p,g,y,m){Ue.prototype.isMatrix4=!0,this.elements=[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],e!==void 0&&this.set(e,t,i,r,s,o,a,l,c,u,d,f,p,g,y,m)}set(e,t,i,r,s,o,a,l,c,u,d,f,p,g,y,m){const h=this.elements;return h[0]=e,h[4]=t,h[8]=i,h[12]=r,h[1]=s,h[5]=o,h[9]=a,h[13]=l,h[2]=c,h[6]=u,h[10]=d,h[14]=f,h[3]=p,h[7]=g,h[11]=y,h[15]=m,this}identity(){return this.set(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1),this}clone(){return new Ue().fromArray(this.elements)}copy(e){const t=this.elements,i=e.elements;return t[0]=i[0],t[1]=i[1],t[2]=i[2],t[3]=i[3],t[4]=i[4],t[5]=i[5],t[6]=i[6],t[7]=i[7],t[8]=i[8],t[9]=i[9],t[10]=i[10],t[11]=i[11],t[12]=i[12],t[13]=i[13],t[14]=i[14],t[15]=i[15],this}copyPosition(e){const t=this.elements,i=e.elements;return t[12]=i[12],t[13]=i[13],t[14]=i[14],this}setFromMatrix3(e){const t=e.elements;return this.set(t[0],t[3],t[6],0,t[1],t[4],t[7],0,t[2],t[5],t[8],0,0,0,0,1),this}extractBasis(e,t,i){return e.setFromMatrixColumn(this,0),t.setFromMatrixColumn(this,1),i.setFromMatrixColumn(this,2),this}makeBasis(e,t,i){return this.set(e.x,t.x,i.x,0,e.y,t.y,i.y,0,e.z,t.z,i.z,0,0,0,0,1),this}extractRotation(e){const t=this.elements,i=e.elements,r=1/Qr.setFromMatrixColumn(e,0).length(),s=1/Qr.setFromMatrixColumn(e,1).length(),o=1/Qr.setFromMatrixColumn(e,2).length();return t[0]=i[0]*r,t[1]=i[1]*r,t[2]=i[2]*r,t[3]=0,t[4]=i[4]*s,t[5]=i[5]*s,t[6]=i[6]*s,t[7]=0,t[8]=i[8]*o,t[9]=i[9]*o,t[10]=i[10]*o,t[11]=0,t[12]=0,t[13]=0,t[14]=0,t[15]=1,this}makeRotationFromEuler(e){const t=this.elements,i=e.x,r=e.y,s=e.z,o=Math.cos(i),a=Math.sin(i),l=Math.cos(r),c=Math.sin(r),u=Math.cos(s),d=Math.sin(s);if(e.order==="XYZ"){const f=o*u,p=o*d,g=a*u,y=a*d;t[0]=l*u,t[4]=-l*d,t[8]=c,t[1]=p+g*c,t[5]=f-y*c,t[9]=-a*l,t[2]=y-f*c,t[6]=g+p*c,t[10]=o*l}else if(e.order==="YXZ"){const f=l*u,p=l*d,g=c*u,y=c*d;t[0]=f+y*a,t[4]=g*a-p,t[8]=o*c,t[1]=o*d,t[5]=o*u,t[9]=-a,t[2]=p*a-g,t[6]=y+f*a,t[10]=o*l}else if(e.order==="ZXY"){const f=l*u,p=l*d,g=c*u,y=c*d;t[0]=f-y*a,t[4]=-o*d,t[8]=g+p*a,t[1]=p+g*a,t[5]=o*u,t[9]=y-f*a,t[2]=-o*c,t[6]=a,t[10]=o*l}else if(e.order==="ZYX"){const f=o*u,p=o*d,g=a*u,y=a*d;t[0]=l*u,t[4]=g*c-p,t[8]=f*c+y,t[1]=l*d,t[5]=y*c+f,t[9]=p*c-g,t[2]=-c,t[6]=a*l,t[10]=o*l}else if(e.order==="YZX"){const f=o*l,p=o*c,g=a*l,y=a*c;t[0]=l*u,t[4]=y-f*d,t[8]=g*d+p,t[1]=d,t[5]=o*u,t[9]=-a*u,t[2]=-c*u,t[6]=p*d+g,t[10]=f-y*d}else if(e.order==="XZY"){const f=o*l,p=o*c,g=a*l,y=a*c;t[0]=l*u,t[4]=-d,t[8]=c*u,t[1]=f*d+y,t[5]=o*u,t[9]=p*d-g,t[2]=g*d-p,t[6]=a*u,t[10]=y*d+f}return t[3]=0,t[7]=0,t[11]=0,t[12]=0,t[13]=0,t[14]=0,t[15]=1,this}makeRotationFromQuaternion(e){return this.compose(xE,e,SE)}lookAt(e,t,i){const r=this.elements;return gn.subVectors(e,t),gn.lengthSq()===0&&(gn.z=1),gn.normalize(),Vi.crossVectors(i,gn),Vi.lengthSq()===0&&(Math.abs(i.z)===1?gn.x+=1e-4:gn.z+=1e-4,gn.normalize(),Vi.crossVectors(i,gn)),Vi.normalize(),Qa.crossVectors(gn,Vi),r[0]=Vi.x,r[4]=Qa.x,r[8]=gn.x,r[1]=Vi.y,r[5]=Qa.y,r[9]=gn.y,r[2]=Vi.z,r[6]=Qa.z,r[10]=gn.z,this}multiply(e){return this.multiplyMatrices(this,e)}premultiply(e){return this.multiplyMatrices(e,this)}multiplyMatrices(e,t){const i=e.elements,r=t.elements,s=this.elements,o=i[0],a=i[4],l=i[8],c=i[12],u=i[1],d=i[5],f=i[9],p=i[13],g=i[2],y=i[6],m=i[10],h=i[14],_=i[3],v=i[7],x=i[11],R=i[15],A=r[0],w=r[4],b=r[8],T=r[12],M=r[1],I=r[5],V=r[9],B=r[13],j=r[2],Y=r[6],W=r[10],$=r[14],N=r[3],K=r[7],Q=r[11],oe=r[15];return s[0]=o*A+a*M+l*j+c*N,s[4]=o*w+a*I+l*Y+c*K,s[8]=o*b+a*V+l*W+c*Q,s[12]=o*T+a*B+l*$+c*oe,s[1]=u*A+d*M+f*j+p*N,s[5]=u*w+d*I+f*Y+p*K,s[9]=u*b+d*V+f*W+p*Q,s[13]=u*T+d*B+f*$+p*oe,s[2]=g*A+y*M+m*j+h*N,s[6]=g*w+y*I+m*Y+h*K,s[10]=g*b+y*V+m*W+h*Q,s[14]=g*T+y*B+m*$+h*oe,s[3]=_*A+v*M+x*j+R*N,s[7]=_*w+v*I+x*Y+R*K,s[11]=_*b+v*V+x*W+R*Q,s[15]=_*T+v*B+x*$+R*oe,this}multiplyScalar(e){const t=this.elements;return t[0]*=e,t[4]*=e,t[8]*=e,t[12]*=e,t[1]*=e,t[5]*=e,t[9]*=e,t[13]*=e,t[2]*=e,t[6]*=e,t[10]*=e,t[14]*=e,t[3]*=e,t[7]*=e,t[11]*=e,t[15]*=e,this}determinant(){const e=this.elements,t=e[0],i=e[4],r=e[8],s=e[12],o=e[1],a=e[5],l=e[9],c=e[13],u=e[2],d=e[6],f=e[10],p=e[14],g=e[3],y=e[7],m=e[11],h=e[15];return g*(+s*l*d-r*c*d-s*a*f+i*c*f+r*a*p-i*l*p)+y*(+t*l*p-t*c*f+s*o*f-r*o*p+r*c*u-s*l*u)+m*(+t*c*d-t*a*p-s*o*d+i*o*p+s*a*u-i*c*u)+h*(-r*a*u-t*l*d+t*a*f+r*o*d-i*o*f+i*l*u)}transpose(){const e=this.elements;let t;return t=e[1],e[1]=e[4],e[4]=t,t=e[2],e[2]=e[8],e[8]=t,t=e[6],e[6]=e[9],e[9]=t,t=e[3],e[3]=e[12],e[12]=t,t=e[7],e[7]=e[13],e[13]=t,t=e[11],e[11]=e[14],e[14]=t,this}setPosition(e,t,i){const r=this.elements;return e.isVector3?(r[12]=e.x,r[13]=e.y,r[14]=e.z):(r[12]=e,r[13]=t,r[14]=i),this}invert(){const e=this.elements,t=e[0],i=e[1],r=e[2],s=e[3],o=e[4],a=e[5],l=e[6],c=e[7],u=e[8],d=e[9],f=e[10],p=e[11],g=e[12],y=e[13],m=e[14],h=e[15],_=d*m*c-y*f*c+y*l*p-a*m*p-d*l*h+a*f*h,v=g*f*c-u*m*c-g*l*p+o*m*p+u*l*h-o*f*h,x=u*y*c-g*d*c+g*a*p-o*y*p-u*a*h+o*d*h,R=g*d*l-u*y*l-g*a*f+o*y*f+u*a*m-o*d*m,A=t*_+i*v+r*x+s*R;if(A===0)return this.set(0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0);const w=1/A;return e[0]=_*w,e[1]=(y*f*s-d*m*s-y*r*p+i*m*p+d*r*h-i*f*h)*w,e[2]=(a*m*s-y*l*s+y*r*c-i*m*c-a*r*h+i*l*h)*w,e[3]=(d*l*s-a*f*s-d*r*c+i*f*c+a*r*p-i*l*p)*w,e[4]=v*w,e[5]=(u*m*s-g*f*s+g*r*p-t*m*p-u*r*h+t*f*h)*w,e[6]=(g*l*s-o*m*s-g*r*c+t*m*c+o*r*h-t*l*h)*w,e[7]=(o*f*s-u*l*s+u*r*c-t*f*c-o*r*p+t*l*p)*w,e[8]=x*w,e[9]=(g*d*s-u*y*s-g*i*p+t*y*p+u*i*h-t*d*h)*w,e[10]=(o*y*s-g*a*s+g*i*c-t*y*c-o*i*h+t*a*h)*w,e[11]=(u*a*s-o*d*s-u*i*c+t*d*c+o*i*p-t*a*p)*w,e[12]=R*w,e[13]=(u*y*r-g*d*r+g*i*f-t*y*f-u*i*m+t*d*m)*w,e[14]=(g*a*r-o*y*r-g*i*l+t*y*l+o*i*m-t*a*m)*w,e[15]=(o*d*r-u*a*r+u*i*l-t*d*l-o*i*f+t*a*f)*w,this}scale(e){const t=this.elements,i=e.x,r=e.y,s=e.z;return t[0]*=i,t[4]*=r,t[8]*=s,t[1]*=i,t[5]*=r,t[9]*=s,t[2]*=i,t[6]*=r,t[10]*=s,t[3]*=i,t[7]*=r,t[11]*=s,this}getMaxScaleOnAxis(){const e=this.elements,t=e[0]*e[0]+e[1]*e[1]+e[2]*e[2],i=e[4]*e[4]+e[5]*e[5]+e[6]*e[6],r=e[8]*e[8]+e[9]*e[9]+e[10]*e[10];return Math.sqrt(Math.max(t,i,r))}makeTranslation(e,t,i){return e.isVector3?this.set(1,0,0,e.x,0,1,0,e.y,0,0,1,e.z,0,0,0,1):this.set(1,0,0,e,0,1,0,t,0,0,1,i,0,0,0,1),this}makeRotationX(e){const t=Math.cos(e),i=Math.sin(e);return this.set(1,0,0,0,0,t,-i,0,0,i,t,0,0,0,0,1),this}makeRotationY(e){const t=Math.cos(e),i=Math.sin(e);return this.set(t,0,i,0,0,1,0,0,-i,0,t,0,0,0,0,1),this}makeRotationZ(e){const t=Math.cos(e),i=Math.sin(e);return this.set(t,-i,0,0,i,t,0,0,0,0,1,0,0,0,0,1),this}makeRotationAxis(e,t){const i=Math.cos(t),r=Math.sin(t),s=1-i,o=e.x,a=e.y,l=e.z,c=s*o,u=s*a;return this.set(c*o+i,c*a-r*l,c*l+r*a,0,c*a+r*l,u*a+i,u*l-r*o,0,c*l-r*a,u*l+r*o,s*l*l+i,0,0,0,0,1),this}makeScale(e,t,i){return this.set(e,0,0,0,0,t,0,0,0,0,i,0,0,0,0,1),this}makeShear(e,t,i,r,s,o){return this.set(1,i,s,0,e,1,o,0,t,r,1,0,0,0,0,1),this}compose(e,t,i){const r=this.elements,s=t._x,o=t._y,a=t._z,l=t._w,c=s+s,u=o+o,d=a+a,f=s*c,p=s*u,g=s*d,y=o*u,m=o*d,h=a*d,_=l*c,v=l*u,x=l*d,R=i.x,A=i.y,w=i.z;return r[0]=(1-(y+h))*R,r[1]=(p+x)*R,r[2]=(g-v)*R,r[3]=0,r[4]=(p-x)*A,r[5]=(1-(f+h))*A,r[6]=(m+_)*A,r[7]=0,r[8]=(g+v)*w,r[9]=(m-_)*w,r[10]=(1-(f+y))*w,r[11]=0,r[12]=e.x,r[13]=e.y,r[14]=e.z,r[15]=1,this}decompose(e,t,i){const r=this.elements;let s=Qr.set(r[0],r[1],r[2]).length();const o=Qr.set(r[4],r[5],r[6]).length(),a=Qr.set(r[8],r[9],r[10]).length();this.determinant()<0&&(s=-s),e.x=r[12],e.y=r[13],e.z=r[14],kn.copy(this);const c=1/s,u=1/o,d=1/a;return kn.elements[0]*=c,kn.elements[1]*=c,kn.elements[2]*=c,kn.elements[4]*=u,kn.elements[5]*=u,kn.elements[6]*=u,kn.elements[8]*=d,kn.elements[9]*=d,kn.elements[10]*=d,t.setFromRotationMatrix(kn),i.x=s,i.y=o,i.z=a,this}makePerspective(e,t,i,r,s,o,a=wi){const l=this.elements,c=2*s/(t-e),u=2*s/(i-r),d=(t+e)/(t-e),f=(i+r)/(i-r);let p,g;if(a===wi)p=-(o+s)/(o-s),g=-2*o*s/(o-s);else if(a===Sc)p=-o/(o-s),g=-o*s/(o-s);else throw new Error("THREE.Matrix4.makePerspective(): Invalid coordinate system: "+a);return l[0]=c,l[4]=0,l[8]=d,l[12]=0,l[1]=0,l[5]=u,l[9]=f,l[13]=0,l[2]=0,l[6]=0,l[10]=p,l[14]=g,l[3]=0,l[7]=0,l[11]=-1,l[15]=0,this}makeOrthographic(e,t,i,r,s,o,a=wi){const l=this.elements,c=1/(t-e),u=1/(i-r),d=1/(o-s),f=(t+e)*c,p=(i+r)*u;let g,y;if(a===wi)g=(o+s)*d,y=-2*d;else if(a===Sc)g=s*d,y=-1*d;else throw new Error("THREE.Matrix4.makeOrthographic(): Invalid coordinate system: "+a);return l[0]=2*c,l[4]=0,l[8]=0,l[12]=-f,l[1]=0,l[5]=2*u,l[9]=0,l[13]=-p,l[2]=0,l[6]=0,l[10]=y,l[14]=-g,l[3]=0,l[7]=0,l[11]=0,l[15]=1,this}equals(e){const t=this.elements,i=e.elements;for(let r=0;r<16;r++)if(t[r]!==i[r])return!1;return!0}fromArray(e,t=0){for(let i=0;i<16;i++)this.elements[i]=e[i+t];return this}toArray(e=[],t=0){const i=this.elements;return e[t]=i[0],e[t+1]=i[1],e[t+2]=i[2],e[t+3]=i[3],e[t+4]=i[4],e[t+5]=i[5],e[t+6]=i[6],e[t+7]=i[7],e[t+8]=i[8],e[t+9]=i[9],e[t+10]=i[10],e[t+11]=i[11],e[t+12]=i[12],e[t+13]=i[13],e[t+14]=i[14],e[t+15]=i[15],e}}const Qr=new L,kn=new Ue,xE=new L(0,0,0),SE=new L(1,1,1),Vi=new L,Qa=new L,gn=new L,Om=new Ue,km=new gr;class ai{constructor(e=0,t=0,i=0,r=ai.DEFAULT_ORDER){this.isEuler=!0,this._x=e,this._y=t,this._z=i,this._order=r}get x(){return this._x}set x(e){this._x=e,this._onChangeCallback()}get y(){return this._y}set y(e){this._y=e,this._onChangeCallback()}get z(){return this._z}set z(e){this._z=e,this._onChangeCallback()}get order(){return this._order}set order(e){this._order=e,this._onChangeCallback()}set(e,t,i,r=this._order){return this._x=e,this._y=t,this._z=i,this._order=r,this._onChangeCallback(),this}clone(){return new this.constructor(this._x,this._y,this._z,this._order)}copy(e){return this._x=e._x,this._y=e._y,this._z=e._z,this._order=e._order,this._onChangeCallback(),this}setFromRotationMatrix(e,t=this._order,i=!0){const r=e.elements,s=r[0],o=r[4],a=r[8],l=r[1],c=r[5],u=r[9],d=r[2],f=r[6],p=r[10];switch(t){case"XYZ":this._y=Math.asin(Ut(a,-1,1)),Math.abs(a)<.9999999?(this._x=Math.atan2(-u,p),this._z=Math.atan2(-o,s)):(this._x=Math.atan2(f,c),this._z=0);break;case"YXZ":this._x=Math.asin(-Ut(u,-1,1)),Math.abs(u)<.9999999?(this._y=Math.atan2(a,p),this._z=Math.atan2(l,c)):(this._y=Math.atan2(-d,s),this._z=0);break;case"ZXY":this._x=Math.asin(Ut(f,-1,1)),Math.abs(f)<.9999999?(this._y=Math.atan2(-d,p),this._z=Math.atan2(-o,c)):(this._y=0,this._z=Math.atan2(l,s));break;case"ZYX":this._y=Math.asin(-Ut(d,-1,1)),Math.abs(d)<.9999999?(this._x=Math.atan2(f,p),this._z=Math.atan2(l,s)):(this._x=0,this._z=Math.atan2(-o,c));break;case"YZX":this._z=Math.asin(Ut(l,-1,1)),Math.abs(l)<.9999999?(this._x=Math.atan2(-u,c),this._y=Math.atan2(-d,s)):(this._x=0,this._y=Math.atan2(a,p));break;case"XZY":this._z=Math.asin(-Ut(o,-1,1)),Math.abs(o)<.9999999?(this._x=Math.atan2(f,c),this._y=Math.atan2(a,s)):(this._x=Math.atan2(-u,p),this._y=0);break;default:console.warn("THREE.Euler: .setFromRotationMatrix() encountered an unknown order: "+t)}return this._order=t,i===!0&&this._onChangeCallback(),this}setFromQuaternion(e,t,i){return Om.makeRotationFromQuaternion(e),this.setFromRotationMatrix(Om,t,i)}setFromVector3(e,t=this._order){return this.set(e.x,e.y,e.z,t)}reorder(e){return km.setFromEuler(this),this.setFromQuaternion(km,e)}equals(e){return e._x===this._x&&e._y===this._y&&e._z===this._z&&e._order===this._order}fromArray(e){return this._x=e[0],this._y=e[1],this._z=e[2],e[3]!==void 0&&(this._order=e[3]),this._onChangeCallback(),this}toArray(e=[],t=0){return e[t]=this._x,e[t+1]=this._y,e[t+2]=this._z,e[t+3]=this._order,e}_onChange(e){return this._onChangeCallback=e,this}_onChangeCallback(){}*[Symbol.iterator](){yield this._x,yield this._y,yield this._z,yield this._order}}ai.DEFAULT_ORDER="XYZ";class Yh{constructor(){this.mask=1}set(e){this.mask=(1<<e|0)>>>0}enable(e){this.mask|=1<<e|0}enableAll(){this.mask=-1}toggle(e){this.mask^=1<<e|0}disable(e){this.mask&=~(1<<e|0)}disableAll(){this.mask=0}test(e){return(this.mask&e.mask)!==0}isEnabled(e){return(this.mask&(1<<e|0))!==0}}let ME=0;const Bm=new L,Jr=new gr,pi=new Ue,Ja=new L,Mo=new L,EE=new L,TE=new gr,zm=new L(1,0,0),Hm=new L(0,1,0),Vm=new L(0,0,1),Gm={type:"added"},wE={type:"removed"},es={type:"childadded",child:null},Lu={type:"childremoved",child:null};class dt extends so{constructor(){super(),this.isObject3D=!0,Object.defineProperty(this,"id",{value:ME++}),this.uuid=Kn(),this.name="",this.type="Object3D",this.parent=null,this.children=[],this.up=dt.DEFAULT_UP.clone();const e=new L,t=new ai,i=new gr,r=new L(1,1,1);function s(){i.setFromEuler(t,!1)}function o(){t.setFromQuaternion(i,void 0,!1)}t._onChange(s),i._onChange(o),Object.defineProperties(this,{position:{configurable:!0,enumerable:!0,value:e},rotation:{configurable:!0,enumerable:!0,value:t},quaternion:{configurable:!0,enumerable:!0,value:i},scale:{configurable:!0,enumerable:!0,value:r},modelViewMatrix:{value:new Ue},normalMatrix:{value:new ze}}),this.matrix=new Ue,this.matrixWorld=new Ue,this.matrixAutoUpdate=dt.DEFAULT_MATRIX_AUTO_UPDATE,this.matrixWorldAutoUpdate=dt.DEFAULT_MATRIX_WORLD_AUTO_UPDATE,this.matrixWorldNeedsUpdate=!1,this.layers=new Yh,this.visible=!0,this.castShadow=!1,this.receiveShadow=!1,this.frustumCulled=!0,this.renderOrder=0,this.animations=[],this.userData={}}onBeforeShadow(){}onAfterShadow(){}onBeforeRender(){}onAfterRender(){}applyMatrix4(e){this.matrixAutoUpdate&&this.updateMatrix(),this.matrix.premultiply(e),this.matrix.decompose(this.position,this.quaternion,this.scale)}applyQuaternion(e){return this.quaternion.premultiply(e),this}setRotationFromAxisAngle(e,t){this.quaternion.setFromAxisAngle(e,t)}setRotationFromEuler(e){this.quaternion.setFromEuler(e,!0)}setRotationFromMatrix(e){this.quaternion.setFromRotationMatrix(e)}setRotationFromQuaternion(e){this.quaternion.copy(e)}rotateOnAxis(e,t){return Jr.setFromAxisAngle(e,t),this.quaternion.multiply(Jr),this}rotateOnWorldAxis(e,t){return Jr.setFromAxisAngle(e,t),this.quaternion.premultiply(Jr),this}rotateX(e){return this.rotateOnAxis(zm,e)}rotateY(e){return this.rotateOnAxis(Hm,e)}rotateZ(e){return this.rotateOnAxis(Vm,e)}translateOnAxis(e,t){return Bm.copy(e).applyQuaternion(this.quaternion),this.position.add(Bm.multiplyScalar(t)),this}translateX(e){return this.translateOnAxis(zm,e)}translateY(e){return this.translateOnAxis(Hm,e)}translateZ(e){return this.translateOnAxis(Vm,e)}localToWorld(e){return this.updateWorldMatrix(!0,!1),e.applyMatrix4(this.matrixWorld)}worldToLocal(e){return this.updateWorldMatrix(!0,!1),e.applyMatrix4(pi.copy(this.matrixWorld).invert())}lookAt(e,t,i){e.isVector3?Ja.copy(e):Ja.set(e,t,i);const r=this.parent;this.updateWorldMatrix(!0,!1),Mo.setFromMatrixPosition(this.matrixWorld),this.isCamera||this.isLight?pi.lookAt(Mo,Ja,this.up):pi.lookAt(Ja,Mo,this.up),this.quaternion.setFromRotationMatrix(pi),r&&(pi.extractRotation(r.matrixWorld),Jr.setFromRotationMatrix(pi),this.quaternion.premultiply(Jr.invert()))}add(e){if(arguments.length>1){for(let t=0;t<arguments.length;t++)this.add(arguments[t]);return this}return e===this?(console.error("THREE.Object3D.add: object can't be added as a child of itself.",e),this):(e&&e.isObject3D?(e.removeFromParent(),e.parent=this,this.children.push(e),e.dispatchEvent(Gm),es.child=e,this.dispatchEvent(es),es.child=null):console.error("THREE.Object3D.add: object not an instance of THREE.Object3D.",e),this)}remove(e){if(arguments.length>1){for(let i=0;i<arguments.length;i++)this.remove(arguments[i]);return this}const t=this.children.indexOf(e);return t!==-1&&(e.parent=null,this.children.splice(t,1),e.dispatchEvent(wE),Lu.child=e,this.dispatchEvent(Lu),Lu.child=null),this}removeFromParent(){const e=this.parent;return e!==null&&e.remove(this),this}clear(){return this.remove(...this.children)}attach(e){return this.updateWorldMatrix(!0,!1),pi.copy(this.matrixWorld).invert(),e.parent!==null&&(e.parent.updateWorldMatrix(!0,!1),pi.multiply(e.parent.matrixWorld)),e.applyMatrix4(pi),e.removeFromParent(),e.parent=this,this.children.push(e),e.updateWorldMatrix(!1,!0),e.dispatchEvent(Gm),es.child=e,this.dispatchEvent(es),es.child=null,this}getObjectById(e){return this.getObjectByProperty("id",e)}getObjectByName(e){return this.getObjectByProperty("name",e)}getObjectByProperty(e,t){if(this[e]===t)return this;for(let i=0,r=this.children.length;i<r;i++){const o=this.children[i].getObjectByProperty(e,t);if(o!==void 0)return o}}getObjectsByProperty(e,t,i=[]){this[e]===t&&i.push(this);const r=this.children;for(let s=0,o=r.length;s<o;s++)r[s].getObjectsByProperty(e,t,i);return i}getWorldPosition(e){return this.updateWorldMatrix(!0,!1),e.setFromMatrixPosition(this.matrixWorld)}getWorldQuaternion(e){return this.updateWorldMatrix(!0,!1),this.matrixWorld.decompose(Mo,e,EE),e}getWorldScale(e){return this.updateWorldMatrix(!0,!1),this.matrixWorld.decompose(Mo,TE,e),e}getWorldDirection(e){this.updateWorldMatrix(!0,!1);const t=this.matrixWorld.elements;return e.set(t[8],t[9],t[10]).normalize()}raycast(){}traverse(e){e(this);const t=this.children;for(let i=0,r=t.length;i<r;i++)t[i].traverse(e)}traverseVisible(e){if(this.visible===!1)return;e(this);const t=this.children;for(let i=0,r=t.length;i<r;i++)t[i].traverseVisible(e)}traverseAncestors(e){const t=this.parent;t!==null&&(e(t),t.traverseAncestors(e))}updateMatrix(){this.matrix.compose(this.position,this.quaternion,this.scale),this.matrixWorldNeedsUpdate=!0}updateMatrixWorld(e){this.matrixAutoUpdate&&this.updateMatrix(),(this.matrixWorldNeedsUpdate||e)&&(this.matrixWorldAutoUpdate===!0&&(this.parent===null?this.matrixWorld.copy(this.matrix):this.matrixWorld.multiplyMatrices(this.parent.matrixWorld,this.matrix)),this.matrixWorldNeedsUpdate=!1,e=!0);const t=this.children;for(let i=0,r=t.length;i<r;i++)t[i].updateMatrixWorld(e)}updateWorldMatrix(e,t){const i=this.parent;if(e===!0&&i!==null&&i.updateWorldMatrix(!0,!1),this.matrixAutoUpdate&&this.updateMatrix(),this.matrixWorldAutoUpdate===!0&&(this.parent===null?this.matrixWorld.copy(this.matrix):this.matrixWorld.multiplyMatrices(this.parent.matrixWorld,this.matrix)),t===!0){const r=this.children;for(let s=0,o=r.length;s<o;s++)r[s].updateWorldMatrix(!1,!0)}}toJSON(e){const t=e===void 0||typeof e=="string",i={};t&&(e={geometries:{},materials:{},textures:{},images:{},shapes:{},skeletons:{},animations:{},nodes:{}},i.metadata={version:4.6,type:"Object",generator:"Object3D.toJSON"});const r={};r.uuid=this.uuid,r.type=this.type,this.name!==""&&(r.name=this.name),this.castShadow===!0&&(r.castShadow=!0),this.receiveShadow===!0&&(r.receiveShadow=!0),this.visible===!1&&(r.visible=!1),this.frustumCulled===!1&&(r.frustumCulled=!1),this.renderOrder!==0&&(r.renderOrder=this.renderOrder),Object.keys(this.userData).length>0&&(r.userData=this.userData),r.layers=this.layers.mask,r.matrix=this.matrix.toArray(),r.up=this.up.toArray(),this.matrixAutoUpdate===!1&&(r.matrixAutoUpdate=!1),this.isInstancedMesh&&(r.type="InstancedMesh",r.count=this.count,r.instanceMatrix=this.instanceMatrix.toJSON(),this.instanceColor!==null&&(r.instanceColor=this.instanceColor.toJSON())),this.isBatchedMesh&&(r.type="BatchedMesh",r.perObjectFrustumCulled=this.perObjectFrustumCulled,r.sortObjects=this.sortObjects,r.drawRanges=this._drawRanges,r.reservedRanges=this._reservedRanges,r.visibility=this._visibility,r.active=this._active,r.bounds=this._bounds.map(a=>({boxInitialized:a.boxInitialized,boxMin:a.box.min.toArray(),boxMax:a.box.max.toArray(),sphereInitialized:a.sphereInitialized,sphereRadius:a.sphere.radius,sphereCenter:a.sphere.center.toArray()})),r.maxInstanceCount=this._maxInstanceCount,r.maxVertexCount=this._maxVertexCount,r.maxIndexCount=this._maxIndexCount,r.geometryInitialized=this._geometryInitialized,r.geometryCount=this._geometryCount,r.matricesTexture=this._matricesTexture.toJSON(e),this._colorsTexture!==null&&(r.colorsTexture=this._colorsTexture.toJSON(e)),this.boundingSphere!==null&&(r.boundingSphere={center:r.boundingSphere.center.toArray(),radius:r.boundingSphere.radius}),this.boundingBox!==null&&(r.boundingBox={min:r.boundingBox.min.toArray(),max:r.boundingBox.max.toArray()}));function s(a,l){return a[l.uuid]===void 0&&(a[l.uuid]=l.toJSON(e)),l.uuid}if(this.isScene)this.background&&(this.background.isColor?r.background=this.background.toJSON():this.background.isTexture&&(r.background=this.background.toJSON(e).uuid)),this.environment&&this.environment.isTexture&&this.environment.isRenderTargetTexture!==!0&&(r.environment=this.environment.toJSON(e).uuid);else if(this.isMesh||this.isLine||this.isPoints){r.geometry=s(e.geometries,this.geometry);const a=this.geometry.parameters;if(a!==void 0&&a.shapes!==void 0){const l=a.shapes;if(Array.isArray(l))for(let c=0,u=l.length;c<u;c++){const d=l[c];s(e.shapes,d)}else s(e.shapes,l)}}if(this.isSkinnedMesh&&(r.bindMode=this.bindMode,r.bindMatrix=this.bindMatrix.toArray(),this.skeleton!==void 0&&(s(e.skeletons,this.skeleton),r.skeleton=this.skeleton.uuid)),this.material!==void 0)if(Array.isArray(this.material)){const a=[];for(let l=0,c=this.material.length;l<c;l++)a.push(s(e.materials,this.material[l]));r.material=a}else r.material=s(e.materials,this.material);if(this.children.length>0){r.children=[];for(let a=0;a<this.children.length;a++)r.children.push(this.children[a].toJSON(e).object)}if(this.animations.length>0){r.animations=[];for(let a=0;a<this.animations.length;a++){const l=this.animations[a];r.animations.push(s(e.animations,l))}}if(t){const a=o(e.geometries),l=o(e.materials),c=o(e.textures),u=o(e.images),d=o(e.shapes),f=o(e.skeletons),p=o(e.animations),g=o(e.nodes);a.length>0&&(i.geometries=a),l.length>0&&(i.materials=l),c.length>0&&(i.textures=c),u.length>0&&(i.images=u),d.length>0&&(i.shapes=d),f.length>0&&(i.skeletons=f),p.length>0&&(i.animations=p),g.length>0&&(i.nodes=g)}return i.object=r,i;function o(a){const l=[];for(const c in a){const u=a[c];delete u.metadata,l.push(u)}return l}}clone(e){return new this.constructor().copy(this,e)}copy(e,t=!0){if(this.name=e.name,this.up.copy(e.up),this.position.copy(e.position),this.rotation.order=e.rotation.order,this.quaternion.copy(e.quaternion),this.scale.copy(e.scale),this.matrix.copy(e.matrix),this.matrixWorld.copy(e.matrixWorld),this.matrixAutoUpdate=e.matrixAutoUpdate,this.matrixWorldAutoUpdate=e.matrixWorldAutoUpdate,this.matrixWorldNeedsUpdate=e.matrixWorldNeedsUpdate,this.layers.mask=e.layers.mask,this.visible=e.visible,this.castShadow=e.castShadow,this.receiveShadow=e.receiveShadow,this.frustumCulled=e.frustumCulled,this.renderOrder=e.renderOrder,this.animations=e.animations.slice(),this.userData=JSON.parse(JSON.stringify(e.userData)),t===!0)for(let i=0;i<e.children.length;i++){const r=e.children[i];this.add(r.clone())}return this}}dt.DEFAULT_UP=new L(0,1,0);dt.DEFAULT_MATRIX_AUTO_UPDATE=!0;dt.DEFAULT_MATRIX_WORLD_AUTO_UPDATE=!0;const Bn=new L,mi=new L,Iu=new L,gi=new L,ts=new L,ns=new L,Wm=new L,Nu=new L,Du=new L,Uu=new L;class Wn{constructor(e=new L,t=new L,i=new L){this.a=e,this.b=t,this.c=i}static getNormal(e,t,i,r){r.subVectors(i,t),Bn.subVectors(e,t),r.cross(Bn);const s=r.lengthSq();return s>0?r.multiplyScalar(1/Math.sqrt(s)):r.set(0,0,0)}static getBarycoord(e,t,i,r,s){Bn.subVectors(r,t),mi.subVectors(i,t),Iu.subVectors(e,t);const o=Bn.dot(Bn),a=Bn.dot(mi),l=Bn.dot(Iu),c=mi.dot(mi),u=mi.dot(Iu),d=o*c-a*a;if(d===0)return s.set(0,0,0),null;const f=1/d,p=(c*l-a*u)*f,g=(o*u-a*l)*f;return s.set(1-p-g,g,p)}static containsPoint(e,t,i,r){return this.getBarycoord(e,t,i,r,gi)===null?!1:gi.x>=0&&gi.y>=0&&gi.x+gi.y<=1}static getInterpolation(e,t,i,r,s,o,a,l){return this.getBarycoord(e,t,i,r,gi)===null?(l.x=0,l.y=0,"z"in l&&(l.z=0),"w"in l&&(l.w=0),null):(l.setScalar(0),l.addScaledVector(s,gi.x),l.addScaledVector(o,gi.y),l.addScaledVector(a,gi.z),l)}static isFrontFacing(e,t,i,r){return Bn.subVectors(i,t),mi.subVectors(e,t),Bn.cross(mi).dot(r)<0}set(e,t,i){return this.a.copy(e),this.b.copy(t),this.c.copy(i),this}setFromPointsAndIndices(e,t,i,r){return this.a.copy(e[t]),this.b.copy(e[i]),this.c.copy(e[r]),this}setFromAttributeAndIndices(e,t,i,r){return this.a.fromBufferAttribute(e,t),this.b.fromBufferAttribute(e,i),this.c.fromBufferAttribute(e,r),this}clone(){return new this.constructor().copy(this)}copy(e){return this.a.copy(e.a),this.b.copy(e.b),this.c.copy(e.c),this}getArea(){return Bn.subVectors(this.c,this.b),mi.subVectors(this.a,this.b),Bn.cross(mi).length()*.5}getMidpoint(e){return e.addVectors(this.a,this.b).add(this.c).multiplyScalar(1/3)}getNormal(e){return Wn.getNormal(this.a,this.b,this.c,e)}getPlane(e){return e.setFromCoplanarPoints(this.a,this.b,this.c)}getBarycoord(e,t){return Wn.getBarycoord(e,this.a,this.b,this.c,t)}getInterpolation(e,t,i,r,s){return Wn.getInterpolation(e,this.a,this.b,this.c,t,i,r,s)}containsPoint(e){return Wn.containsPoint(e,this.a,this.b,this.c)}isFrontFacing(e){return Wn.isFrontFacing(this.a,this.b,this.c,e)}intersectsBox(e){return e.intersectsTriangle(this)}closestPointToPoint(e,t){const i=this.a,r=this.b,s=this.c;let o,a;ts.subVectors(r,i),ns.subVectors(s,i),Nu.subVectors(e,i);const l=ts.dot(Nu),c=ns.dot(Nu);if(l<=0&&c<=0)return t.copy(i);Du.subVectors(e,r);const u=ts.dot(Du),d=ns.dot(Du);if(u>=0&&d<=u)return t.copy(r);const f=l*d-u*c;if(f<=0&&l>=0&&u<=0)return o=l/(l-u),t.copy(i).addScaledVector(ts,o);Uu.subVectors(e,s);const p=ts.dot(Uu),g=ns.dot(Uu);if(g>=0&&p<=g)return t.copy(s);const y=p*c-l*g;if(y<=0&&c>=0&&g<=0)return a=c/(c-g),t.copy(i).addScaledVector(ns,a);const m=u*g-p*d;if(m<=0&&d-u>=0&&p-g>=0)return Wm.subVectors(s,r),a=(d-u)/(d-u+(p-g)),t.copy(r).addScaledVector(Wm,a);const h=1/(m+y+f);return o=y*h,a=f*h,t.copy(i).addScaledVector(ts,o).addScaledVector(ns,a)}equals(e){return e.a.equals(this.a)&&e.b.equals(this.b)&&e.c.equals(this.c)}}const Fv={aliceblue:15792383,antiquewhite:16444375,aqua:65535,aquamarine:8388564,azure:15794175,beige:16119260,bisque:16770244,black:0,blanchedalmond:16772045,blue:255,blueviolet:9055202,brown:10824234,burlywood:14596231,cadetblue:6266528,chartreuse:8388352,chocolate:13789470,coral:16744272,cornflowerblue:6591981,cornsilk:16775388,crimson:14423100,cyan:65535,darkblue:139,darkcyan:35723,darkgoldenrod:12092939,darkgray:11119017,darkgreen:25600,darkgrey:11119017,darkkhaki:12433259,darkmagenta:9109643,darkolivegreen:5597999,darkorange:16747520,darkorchid:10040012,darkred:9109504,darksalmon:15308410,darkseagreen:9419919,darkslateblue:4734347,darkslategray:3100495,darkslategrey:3100495,darkturquoise:52945,darkviolet:9699539,deeppink:16716947,deepskyblue:49151,dimgray:6908265,dimgrey:6908265,dodgerblue:2003199,firebrick:11674146,floralwhite:16775920,forestgreen:2263842,fuchsia:16711935,gainsboro:14474460,ghostwhite:16316671,gold:16766720,goldenrod:14329120,gray:8421504,green:32768,greenyellow:11403055,grey:8421504,honeydew:15794160,hotpink:16738740,indianred:13458524,indigo:4915330,ivory:16777200,khaki:15787660,lavender:15132410,lavenderblush:16773365,lawngreen:8190976,lemonchiffon:16775885,lightblue:11393254,lightcoral:15761536,lightcyan:14745599,lightgoldenrodyellow:16448210,lightgray:13882323,lightgreen:9498256,lightgrey:13882323,lightpink:16758465,lightsalmon:16752762,lightseagreen:2142890,lightskyblue:8900346,lightslategray:7833753,lightslategrey:7833753,lightsteelblue:11584734,lightyellow:16777184,lime:65280,limegreen:3329330,linen:16445670,magenta:16711935,maroon:8388608,mediumaquamarine:6737322,mediumblue:205,mediumorchid:12211667,mediumpurple:9662683,mediumseagreen:3978097,mediumslateblue:8087790,mediumspringgreen:64154,mediumturquoise:4772300,mediumvioletred:13047173,midnightblue:1644912,mintcream:16121850,mistyrose:16770273,moccasin:16770229,navajowhite:16768685,navy:128,oldlace:16643558,olive:8421376,olivedrab:7048739,orange:16753920,orangered:16729344,orchid:14315734,palegoldenrod:15657130,palegreen:10025880,paleturquoise:11529966,palevioletred:14381203,papayawhip:16773077,peachpuff:16767673,peru:13468991,pink:16761035,plum:14524637,powderblue:11591910,purple:8388736,rebeccapurple:6697881,red:16711680,rosybrown:12357519,royalblue:4286945,saddlebrown:9127187,salmon:16416882,sandybrown:16032864,seagreen:3050327,seashell:16774638,sienna:10506797,silver:12632256,skyblue:8900331,slateblue:6970061,slategray:7372944,slategrey:7372944,snow:16775930,springgreen:65407,steelblue:4620980,tan:13808780,teal:32896,thistle:14204888,tomato:16737095,turquoise:4251856,violet:15631086,wheat:16113331,white:16777215,whitesmoke:16119285,yellow:16776960,yellowgreen:10145074},Gi={h:0,s:0,l:0},el={h:0,s:0,l:0};function Fu(n,e,t){return t<0&&(t+=1),t>1&&(t-=1),t<1/6?n+(e-n)*6*t:t<1/2?e:t<2/3?n+(e-n)*6*(2/3-t):n}class Pe{constructor(e,t,i){return this.isColor=!0,this.r=1,this.g=1,this.b=1,this.set(e,t,i)}set(e,t,i){if(t===void 0&&i===void 0){const r=e;r&&r.isColor?this.copy(r):typeof r=="number"?this.setHex(r):typeof r=="string"&&this.setStyle(r)}else this.setRGB(e,t,i);return this}setScalar(e){return this.r=e,this.g=e,this.b=e,this}setHex(e,t=Qt){return e=Math.floor(e),this.r=(e>>16&255)/255,this.g=(e>>8&255)/255,this.b=(e&255)/255,qe.toWorkingColorSpace(this,t),this}setRGB(e,t,i,r=qe.workingColorSpace){return this.r=e,this.g=t,this.b=i,qe.toWorkingColorSpace(this,r),this}setHSL(e,t,i,r=qe.workingColorSpace){if(e=jh(e,1),t=Ut(t,0,1),i=Ut(i,0,1),t===0)this.r=this.g=this.b=i;else{const s=i<=.5?i*(1+t):i+t-i*t,o=2*i-s;this.r=Fu(o,s,e+1/3),this.g=Fu(o,s,e),this.b=Fu(o,s,e-1/3)}return qe.toWorkingColorSpace(this,r),this}setStyle(e,t=Qt){function i(s){s!==void 0&&parseFloat(s)<1&&console.warn("THREE.Color: Alpha component of "+e+" will be ignored.")}let r;if(r=/^(\w+)\(([^\)]*)\)/.exec(e)){let s;const o=r[1],a=r[2];switch(o){case"rgb":case"rgba":if(s=/^\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(a))return i(s[4]),this.setRGB(Math.min(255,parseInt(s[1],10))/255,Math.min(255,parseInt(s[2],10))/255,Math.min(255,parseInt(s[3],10))/255,t);if(s=/^\s*(\d+)\%\s*,\s*(\d+)\%\s*,\s*(\d+)\%\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(a))return i(s[4]),this.setRGB(Math.min(100,parseInt(s[1],10))/100,Math.min(100,parseInt(s[2],10))/100,Math.min(100,parseInt(s[3],10))/100,t);break;case"hsl":case"hsla":if(s=/^\s*(\d*\.?\d+)\s*,\s*(\d*\.?\d+)\%\s*,\s*(\d*\.?\d+)\%\s*(?:,\s*(\d*\.?\d+)\s*)?$/.exec(a))return i(s[4]),this.setHSL(parseFloat(s[1])/360,parseFloat(s[2])/100,parseFloat(s[3])/100,t);break;default:console.warn("THREE.Color: Unknown color model "+e)}}else if(r=/^\#([A-Fa-f\d]+)$/.exec(e)){const s=r[1],o=s.length;if(o===3)return this.setRGB(parseInt(s.charAt(0),16)/15,parseInt(s.charAt(1),16)/15,parseInt(s.charAt(2),16)/15,t);if(o===6)return this.setHex(parseInt(s,16),t);console.warn("THREE.Color: Invalid hex color "+e)}else if(e&&e.length>0)return this.setColorName(e,t);return this}setColorName(e,t=Qt){const i=Fv[e.toLowerCase()];return i!==void 0?this.setHex(i,t):console.warn("THREE.Color: Unknown color "+e),this}clone(){return new this.constructor(this.r,this.g,this.b)}copy(e){return this.r=e.r,this.g=e.g,this.b=e.b,this}copySRGBToLinear(e){return this.r=Os(e.r),this.g=Os(e.g),this.b=Os(e.b),this}copyLinearToSRGB(e){return this.r=Eu(e.r),this.g=Eu(e.g),this.b=Eu(e.b),this}convertSRGBToLinear(){return this.copySRGBToLinear(this),this}convertLinearToSRGB(){return this.copyLinearToSRGB(this),this}getHex(e=Qt){return qe.fromWorkingColorSpace(jt.copy(this),e),Math.round(Ut(jt.r*255,0,255))*65536+Math.round(Ut(jt.g*255,0,255))*256+Math.round(Ut(jt.b*255,0,255))}getHexString(e=Qt){return("000000"+this.getHex(e).toString(16)).slice(-6)}getHSL(e,t=qe.workingColorSpace){qe.fromWorkingColorSpace(jt.copy(this),t);const i=jt.r,r=jt.g,s=jt.b,o=Math.max(i,r,s),a=Math.min(i,r,s);let l,c;const u=(a+o)/2;if(a===o)l=0,c=0;else{const d=o-a;switch(c=u<=.5?d/(o+a):d/(2-o-a),o){case i:l=(r-s)/d+(r<s?6:0);break;case r:l=(s-i)/d+2;break;case s:l=(i-r)/d+4;break}l/=6}return e.h=l,e.s=c,e.l=u,e}getRGB(e,t=qe.workingColorSpace){return qe.fromWorkingColorSpace(jt.copy(this),t),e.r=jt.r,e.g=jt.g,e.b=jt.b,e}getStyle(e=Qt){qe.fromWorkingColorSpace(jt.copy(this),e);const t=jt.r,i=jt.g,r=jt.b;return e!==Qt?`color(${e} ${t.toFixed(3)} ${i.toFixed(3)} ${r.toFixed(3)})`:`rgb(${Math.round(t*255)},${Math.round(i*255)},${Math.round(r*255)})`}offsetHSL(e,t,i){return this.getHSL(Gi),this.setHSL(Gi.h+e,Gi.s+t,Gi.l+i)}add(e){return this.r+=e.r,this.g+=e.g,this.b+=e.b,this}addColors(e,t){return this.r=e.r+t.r,this.g=e.g+t.g,this.b=e.b+t.b,this}addScalar(e){return this.r+=e,this.g+=e,this.b+=e,this}sub(e){return this.r=Math.max(0,this.r-e.r),this.g=Math.max(0,this.g-e.g),this.b=Math.max(0,this.b-e.b),this}multiply(e){return this.r*=e.r,this.g*=e.g,this.b*=e.b,this}multiplyScalar(e){return this.r*=e,this.g*=e,this.b*=e,this}lerp(e,t){return this.r+=(e.r-this.r)*t,this.g+=(e.g-this.g)*t,this.b+=(e.b-this.b)*t,this}lerpColors(e,t,i){return this.r=e.r+(t.r-e.r)*i,this.g=e.g+(t.g-e.g)*i,this.b=e.b+(t.b-e.b)*i,this}lerpHSL(e,t){this.getHSL(Gi),e.getHSL(el);const i=Ko(Gi.h,el.h,t),r=Ko(Gi.s,el.s,t),s=Ko(Gi.l,el.l,t);return this.setHSL(i,r,s),this}setFromVector3(e){return this.r=e.x,this.g=e.y,this.b=e.z,this}applyMatrix3(e){const t=this.r,i=this.g,r=this.b,s=e.elements;return this.r=s[0]*t+s[3]*i+s[6]*r,this.g=s[1]*t+s[4]*i+s[7]*r,this.b=s[2]*t+s[5]*i+s[8]*r,this}equals(e){return e.r===this.r&&e.g===this.g&&e.b===this.b}fromArray(e,t=0){return this.r=e[t],this.g=e[t+1],this.b=e[t+2],this}toArray(e=[],t=0){return e[t]=this.r,e[t+1]=this.g,e[t+2]=this.b,e}fromBufferAttribute(e,t){return this.r=e.getX(t),this.g=e.getY(t),this.b=e.getZ(t),this}toJSON(){return this.getHex()}*[Symbol.iterator](){yield this.r,yield this.g,yield this.b}}const jt=new Pe;Pe.NAMES=Fv;let AE=0;class qn extends so{constructor(){super(),this.isMaterial=!0,Object.defineProperty(this,"id",{value:AE++}),this.uuid=Kn(),this.name="",this.type="Material",this.blending=Ds,this.side=Ii,this.vertexColors=!1,this.opacity=1,this.transparent=!1,this.alphaHash=!1,this.blendSrc=nf,this.blendDst=rf,this.blendEquation=br,this.blendSrcAlpha=null,this.blendDstAlpha=null,this.blendEquationAlpha=null,this.blendColor=new Pe(0,0,0),this.blendAlpha=0,this.depthFunc=gc,this.depthTest=!0,this.depthWrite=!0,this.stencilWriteMask=255,this.stencilFunc=Pm,this.stencilRef=0,this.stencilFuncMask=255,this.stencilFail=Yr,this.stencilZFail=Yr,this.stencilZPass=Yr,this.stencilWrite=!1,this.clippingPlanes=null,this.clipIntersection=!1,this.clipShadows=!1,this.shadowSide=null,this.colorWrite=!0,this.precision=null,this.polygonOffset=!1,this.polygonOffsetFactor=0,this.polygonOffsetUnits=0,this.dithering=!1,this.alphaToCoverage=!1,this.premultipliedAlpha=!1,this.forceSinglePass=!1,this.visible=!0,this.toneMapped=!0,this.userData={},this.version=0,this._alphaTest=0}get alphaTest(){return this._alphaTest}set alphaTest(e){this._alphaTest>0!=e>0&&this.version++,this._alphaTest=e}onBeforeRender(){}onBeforeCompile(){}customProgramCacheKey(){return this.onBeforeCompile.toString()}setValues(e){if(e!==void 0)for(const t in e){const i=e[t];if(i===void 0){console.warn(`THREE.Material: parameter '${t}' has value of undefined.`);continue}const r=this[t];if(r===void 0){console.warn(`THREE.Material: '${t}' is not a property of THREE.${this.type}.`);continue}r&&r.isColor?r.set(i):r&&r.isVector3&&i&&i.isVector3?r.copy(i):this[t]=i}}toJSON(e){const t=e===void 0||typeof e=="string";t&&(e={textures:{},images:{}});const i={metadata:{version:4.6,type:"Material",generator:"Material.toJSON"}};i.uuid=this.uuid,i.type=this.type,this.name!==""&&(i.name=this.name),this.color&&this.color.isColor&&(i.color=this.color.getHex()),this.roughness!==void 0&&(i.roughness=this.roughness),this.metalness!==void 0&&(i.metalness=this.metalness),this.sheen!==void 0&&(i.sheen=this.sheen),this.sheenColor&&this.sheenColor.isColor&&(i.sheenColor=this.sheenColor.getHex()),this.sheenRoughness!==void 0&&(i.sheenRoughness=this.sheenRoughness),this.emissive&&this.emissive.isColor&&(i.emissive=this.emissive.getHex()),this.emissiveIntensity!==void 0&&this.emissiveIntensity!==1&&(i.emissiveIntensity=this.emissiveIntensity),this.specular&&this.specular.isColor&&(i.specular=this.specular.getHex()),this.specularIntensity!==void 0&&(i.specularIntensity=this.specularIntensity),this.specularColor&&this.specularColor.isColor&&(i.specularColor=this.specularColor.getHex()),this.shininess!==void 0&&(i.shininess=this.shininess),this.clearcoat!==void 0&&(i.clearcoat=this.clearcoat),this.clearcoatRoughness!==void 0&&(i.clearcoatRoughness=this.clearcoatRoughness),this.clearcoatMap&&this.clearcoatMap.isTexture&&(i.clearcoatMap=this.clearcoatMap.toJSON(e).uuid),this.clearcoatRoughnessMap&&this.clearcoatRoughnessMap.isTexture&&(i.clearcoatRoughnessMap=this.clearcoatRoughnessMap.toJSON(e).uuid),this.clearcoatNormalMap&&this.clearcoatNormalMap.isTexture&&(i.clearcoatNormalMap=this.clearcoatNormalMap.toJSON(e).uuid,i.clearcoatNormalScale=this.clearcoatNormalScale.toArray()),this.dispersion!==void 0&&(i.dispersion=this.dispersion),this.iridescence!==void 0&&(i.iridescence=this.iridescence),this.iridescenceIOR!==void 0&&(i.iridescenceIOR=this.iridescenceIOR),this.iridescenceThicknessRange!==void 0&&(i.iridescenceThicknessRange=this.iridescenceThicknessRange),this.iridescenceMap&&this.iridescenceMap.isTexture&&(i.iridescenceMap=this.iridescenceMap.toJSON(e).uuid),this.iridescenceThicknessMap&&this.iridescenceThicknessMap.isTexture&&(i.iridescenceThicknessMap=this.iridescenceThicknessMap.toJSON(e).uuid),this.anisotropy!==void 0&&(i.anisotropy=this.anisotropy),this.anisotropyRotation!==void 0&&(i.anisotropyRotation=this.anisotropyRotation),this.anisotropyMap&&this.anisotropyMap.isTexture&&(i.anisotropyMap=this.anisotropyMap.toJSON(e).uuid),this.map&&this.map.isTexture&&(i.map=this.map.toJSON(e).uuid),this.matcap&&this.matcap.isTexture&&(i.matcap=this.matcap.toJSON(e).uuid),this.alphaMap&&this.alphaMap.isTexture&&(i.alphaMap=this.alphaMap.toJSON(e).uuid),this.lightMap&&this.lightMap.isTexture&&(i.lightMap=this.lightMap.toJSON(e).uuid,i.lightMapIntensity=this.lightMapIntensity),this.aoMap&&this.aoMap.isTexture&&(i.aoMap=this.aoMap.toJSON(e).uuid,i.aoMapIntensity=this.aoMapIntensity),this.bumpMap&&this.bumpMap.isTexture&&(i.bumpMap=this.bumpMap.toJSON(e).uuid,i.bumpScale=this.bumpScale),this.normalMap&&this.normalMap.isTexture&&(i.normalMap=this.normalMap.toJSON(e).uuid,i.normalMapType=this.normalMapType,i.normalScale=this.normalScale.toArray()),this.displacementMap&&this.displacementMap.isTexture&&(i.displacementMap=this.displacementMap.toJSON(e).uuid,i.displacementScale=this.displacementScale,i.displacementBias=this.displacementBias),this.roughnessMap&&this.roughnessMap.isTexture&&(i.roughnessMap=this.roughnessMap.toJSON(e).uuid),this.metalnessMap&&this.metalnessMap.isTexture&&(i.metalnessMap=this.metalnessMap.toJSON(e).uuid),this.emissiveMap&&this.emissiveMap.isTexture&&(i.emissiveMap=this.emissiveMap.toJSON(e).uuid),this.specularMap&&this.specularMap.isTexture&&(i.specularMap=this.specularMap.toJSON(e).uuid),this.specularIntensityMap&&this.specularIntensityMap.isTexture&&(i.specularIntensityMap=this.specularIntensityMap.toJSON(e).uuid),this.specularColorMap&&this.specularColorMap.isTexture&&(i.specularColorMap=this.specularColorMap.toJSON(e).uuid),this.envMap&&this.envMap.isTexture&&(i.envMap=this.envMap.toJSON(e).uuid,this.combine!==void 0&&(i.combine=this.combine)),this.envMapRotation!==void 0&&(i.envMapRotation=this.envMapRotation.toArray()),this.envMapIntensity!==void 0&&(i.envMapIntensity=this.envMapIntensity),this.reflectivity!==void 0&&(i.reflectivity=this.reflectivity),this.refractionRatio!==void 0&&(i.refractionRatio=this.refractionRatio),this.gradientMap&&this.gradientMap.isTexture&&(i.gradientMap=this.gradientMap.toJSON(e).uuid),this.transmission!==void 0&&(i.transmission=this.transmission),this.transmissionMap&&this.transmissionMap.isTexture&&(i.transmissionMap=this.transmissionMap.toJSON(e).uuid),this.thickness!==void 0&&(i.thickness=this.thickness),this.thicknessMap&&this.thicknessMap.isTexture&&(i.thicknessMap=this.thicknessMap.toJSON(e).uuid),this.attenuationDistance!==void 0&&this.attenuationDistance!==1/0&&(i.attenuationDistance=this.attenuationDistance),this.attenuationColor!==void 0&&(i.attenuationColor=this.attenuationColor.getHex()),this.size!==void 0&&(i.size=this.size),this.shadowSide!==null&&(i.shadowSide=this.shadowSide),this.sizeAttenuation!==void 0&&(i.sizeAttenuation=this.sizeAttenuation),this.blending!==Ds&&(i.blending=this.blending),this.side!==Ii&&(i.side=this.side),this.vertexColors===!0&&(i.vertexColors=!0),this.opacity<1&&(i.opacity=this.opacity),this.transparent===!0&&(i.transparent=!0),this.blendSrc!==nf&&(i.blendSrc=this.blendSrc),this.blendDst!==rf&&(i.blendDst=this.blendDst),this.blendEquation!==br&&(i.blendEquation=this.blendEquation),this.blendSrcAlpha!==null&&(i.blendSrcAlpha=this.blendSrcAlpha),this.blendDstAlpha!==null&&(i.blendDstAlpha=this.blendDstAlpha),this.blendEquationAlpha!==null&&(i.blendEquationAlpha=this.blendEquationAlpha),this.blendColor&&this.blendColor.isColor&&(i.blendColor=this.blendColor.getHex()),this.blendAlpha!==0&&(i.blendAlpha=this.blendAlpha),this.depthFunc!==gc&&(i.depthFunc=this.depthFunc),this.depthTest===!1&&(i.depthTest=this.depthTest),this.depthWrite===!1&&(i.depthWrite=this.depthWrite),this.colorWrite===!1&&(i.colorWrite=this.colorWrite),this.stencilWriteMask!==255&&(i.stencilWriteMask=this.stencilWriteMask),this.stencilFunc!==Pm&&(i.stencilFunc=this.stencilFunc),this.stencilRef!==0&&(i.stencilRef=this.stencilRef),this.stencilFuncMask!==255&&(i.stencilFuncMask=this.stencilFuncMask),this.stencilFail!==Yr&&(i.stencilFail=this.stencilFail),this.stencilZFail!==Yr&&(i.stencilZFail=this.stencilZFail),this.stencilZPass!==Yr&&(i.stencilZPass=this.stencilZPass),this.stencilWrite===!0&&(i.stencilWrite=this.stencilWrite),this.rotation!==void 0&&this.rotation!==0&&(i.rotation=this.rotation),this.polygonOffset===!0&&(i.polygonOffset=!0),this.polygonOffsetFactor!==0&&(i.polygonOffsetFactor=this.polygonOffsetFactor),this.polygonOffsetUnits!==0&&(i.polygonOffsetUnits=this.polygonOffsetUnits),this.linewidth!==void 0&&this.linewidth!==1&&(i.linewidth=this.linewidth),this.dashSize!==void 0&&(i.dashSize=this.dashSize),this.gapSize!==void 0&&(i.gapSize=this.gapSize),this.scale!==void 0&&(i.scale=this.scale),this.dithering===!0&&(i.dithering=!0),this.alphaTest>0&&(i.alphaTest=this.alphaTest),this.alphaHash===!0&&(i.alphaHash=!0),this.alphaToCoverage===!0&&(i.alphaToCoverage=!0),this.premultipliedAlpha===!0&&(i.premultipliedAlpha=!0),this.forceSinglePass===!0&&(i.forceSinglePass=!0),this.wireframe===!0&&(i.wireframe=!0),this.wireframeLinewidth>1&&(i.wireframeLinewidth=this.wireframeLinewidth),this.wireframeLinecap!=="round"&&(i.wireframeLinecap=this.wireframeLinecap),this.wireframeLinejoin!=="round"&&(i.wireframeLinejoin=this.wireframeLinejoin),this.flatShading===!0&&(i.flatShading=!0),this.visible===!1&&(i.visible=!1),this.toneMapped===!1&&(i.toneMapped=!1),this.fog===!1&&(i.fog=!1),Object.keys(this.userData).length>0&&(i.userData=this.userData);function r(s){const o=[];for(const a in s){const l=s[a];delete l.metadata,o.push(l)}return o}if(t){const s=r(e.textures),o=r(e.images);s.length>0&&(i.textures=s),o.length>0&&(i.images=o)}return i}clone(){return new this.constructor().copy(this)}copy(e){this.name=e.name,this.blending=e.blending,this.side=e.side,this.vertexColors=e.vertexColors,this.opacity=e.opacity,this.transparent=e.transparent,this.blendSrc=e.blendSrc,this.blendDst=e.blendDst,this.blendEquation=e.blendEquation,this.blendSrcAlpha=e.blendSrcAlpha,this.blendDstAlpha=e.blendDstAlpha,this.blendEquationAlpha=e.blendEquationAlpha,this.blendColor.copy(e.blendColor),this.blendAlpha=e.blendAlpha,this.depthFunc=e.depthFunc,this.depthTest=e.depthTest,this.depthWrite=e.depthWrite,this.stencilWriteMask=e.stencilWriteMask,this.stencilFunc=e.stencilFunc,this.stencilRef=e.stencilRef,this.stencilFuncMask=e.stencilFuncMask,this.stencilFail=e.stencilFail,this.stencilZFail=e.stencilZFail,this.stencilZPass=e.stencilZPass,this.stencilWrite=e.stencilWrite;const t=e.clippingPlanes;let i=null;if(t!==null){const r=t.length;i=new Array(r);for(let s=0;s!==r;++s)i[s]=t[s].clone()}return this.clippingPlanes=i,this.clipIntersection=e.clipIntersection,this.clipShadows=e.clipShadows,this.shadowSide=e.shadowSide,this.colorWrite=e.colorWrite,this.precision=e.precision,this.polygonOffset=e.polygonOffset,this.polygonOffsetFactor=e.polygonOffsetFactor,this.polygonOffsetUnits=e.polygonOffsetUnits,this.dithering=e.dithering,this.alphaTest=e.alphaTest,this.alphaHash=e.alphaHash,this.alphaToCoverage=e.alphaToCoverage,this.premultipliedAlpha=e.premultipliedAlpha,this.forceSinglePass=e.forceSinglePass,this.visible=e.visible,this.toneMapped=e.toneMapped,this.userData=JSON.parse(JSON.stringify(e.userData)),this}dispose(){this.dispatchEvent({type:"dispose"})}set needsUpdate(e){e===!0&&this.version++}onBuild(){console.warn("Material: onBuild() has been removed.")}}class Ai extends qn{constructor(e){super(),this.isMeshBasicMaterial=!0,this.type="MeshBasicMaterial",this.color=new Pe(16777215),this.map=null,this.lightMap=null,this.lightMapIntensity=1,this.aoMap=null,this.aoMapIntensity=1,this.specularMap=null,this.alphaMap=null,this.envMap=null,this.envMapRotation=new ai,this.combine=vv,this.reflectivity=1,this.refractionRatio=.98,this.wireframe=!1,this.wireframeLinewidth=1,this.wireframeLinecap="round",this.wireframeLinejoin="round",this.fog=!0,this.setValues(e)}copy(e){return super.copy(e),this.color.copy(e.color),this.map=e.map,this.lightMap=e.lightMap,this.lightMapIntensity=e.lightMapIntensity,this.aoMap=e.aoMap,this.aoMapIntensity=e.aoMapIntensity,this.specularMap=e.specularMap,this.alphaMap=e.alphaMap,this.envMap=e.envMap,this.envMapRotation.copy(e.envMapRotation),this.combine=e.combine,this.reflectivity=e.reflectivity,this.refractionRatio=e.refractionRatio,this.wireframe=e.wireframe,this.wireframeLinewidth=e.wireframeLinewidth,this.wireframeLinecap=e.wireframeLinecap,this.wireframeLinejoin=e.wireframeLinejoin,this.fog=e.fog,this}}const wt=new L,tl=new we;class Rt{constructor(e,t,i=!1){if(Array.isArray(e))throw new TypeError("THREE.BufferAttribute: array should be a Typed Array.");this.isBufferAttribute=!0,this.name="",this.array=e,this.itemSize=t,this.count=e!==void 0?e.length/t:0,this.normalized=i,this.usage=Df,this._updateRange={offset:0,count:-1},this.updateRanges=[],this.gpuType=Xn,this.version=0}onUploadCallback(){}set needsUpdate(e){e===!0&&this.version++}get updateRange(){return Fs("THREE.BufferAttribute: updateRange() is deprecated and will be removed in r169. Use addUpdateRange() instead."),this._updateRange}setUsage(e){return this.usage=e,this}addUpdateRange(e,t){this.updateRanges.push({start:e,count:t})}clearUpdateRanges(){this.updateRanges.length=0}copy(e){return this.name=e.name,this.array=new e.array.constructor(e.array),this.itemSize=e.itemSize,this.count=e.count,this.normalized=e.normalized,this.usage=e.usage,this.gpuType=e.gpuType,this}copyAt(e,t,i){e*=this.itemSize,i*=t.itemSize;for(let r=0,s=this.itemSize;r<s;r++)this.array[e+r]=t.array[i+r];return this}copyArray(e){return this.array.set(e),this}applyMatrix3(e){if(this.itemSize===2)for(let t=0,i=this.count;t<i;t++)tl.fromBufferAttribute(this,t),tl.applyMatrix3(e),this.setXY(t,tl.x,tl.y);else if(this.itemSize===3)for(let t=0,i=this.count;t<i;t++)wt.fromBufferAttribute(this,t),wt.applyMatrix3(e),this.setXYZ(t,wt.x,wt.y,wt.z);return this}applyMatrix4(e){for(let t=0,i=this.count;t<i;t++)wt.fromBufferAttribute(this,t),wt.applyMatrix4(e),this.setXYZ(t,wt.x,wt.y,wt.z);return this}applyNormalMatrix(e){for(let t=0,i=this.count;t<i;t++)wt.fromBufferAttribute(this,t),wt.applyNormalMatrix(e),this.setXYZ(t,wt.x,wt.y,wt.z);return this}transformDirection(e){for(let t=0,i=this.count;t<i;t++)wt.fromBufferAttribute(this,t),wt.transformDirection(e),this.setXYZ(t,wt.x,wt.y,wt.z);return this}set(e,t=0){return this.array.set(e,t),this}getComponent(e,t){let i=this.array[e*this.itemSize+t];return this.normalized&&(i=Gn(i,this.array)),i}setComponent(e,t,i){return this.normalized&&(i=tt(i,this.array)),this.array[e*this.itemSize+t]=i,this}getX(e){let t=this.array[e*this.itemSize];return this.normalized&&(t=Gn(t,this.array)),t}setX(e,t){return this.normalized&&(t=tt(t,this.array)),this.array[e*this.itemSize]=t,this}getY(e){let t=this.array[e*this.itemSize+1];return this.normalized&&(t=Gn(t,this.array)),t}setY(e,t){return this.normalized&&(t=tt(t,this.array)),this.array[e*this.itemSize+1]=t,this}getZ(e){let t=this.array[e*this.itemSize+2];return this.normalized&&(t=Gn(t,this.array)),t}setZ(e,t){return this.normalized&&(t=tt(t,this.array)),this.array[e*this.itemSize+2]=t,this}getW(e){let t=this.array[e*this.itemSize+3];return this.normalized&&(t=Gn(t,this.array)),t}setW(e,t){return this.normalized&&(t=tt(t,this.array)),this.array[e*this.itemSize+3]=t,this}setXY(e,t,i){return e*=this.itemSize,this.normalized&&(t=tt(t,this.array),i=tt(i,this.array)),this.array[e+0]=t,this.array[e+1]=i,this}setXYZ(e,t,i,r){return e*=this.itemSize,this.normalized&&(t=tt(t,this.array),i=tt(i,this.array),r=tt(r,this.array)),this.array[e+0]=t,this.array[e+1]=i,this.array[e+2]=r,this}setXYZW(e,t,i,r,s){return e*=this.itemSize,this.normalized&&(t=tt(t,this.array),i=tt(i,this.array),r=tt(r,this.array),s=tt(s,this.array)),this.array[e+0]=t,this.array[e+1]=i,this.array[e+2]=r,this.array[e+3]=s,this}onUpload(e){return this.onUploadCallback=e,this}clone(){return new this.constructor(this.array,this.itemSize).copy(this)}toJSON(){const e={itemSize:this.itemSize,type:this.array.constructor.name,array:Array.from(this.array),normalized:this.normalized};return this.name!==""&&(e.name=this.name),this.usage!==Df&&(e.usage=this.usage),e}}class Ov extends Rt{constructor(e,t,i){super(new Uint16Array(e),t,i)}}class kv extends Rt{constructor(e,t,i){super(new Uint32Array(e),t,i)}}class In extends Rt{constructor(e,t,i){super(new Float32Array(e),t,i)}}let RE=0;const wn=new Ue,Ou=new dt,is=new L,_n=new Fi,Eo=new Fi,Nt=new L;class hn extends so{constructor(){super(),this.isBufferGeometry=!0,Object.defineProperty(this,"id",{value:RE++}),this.uuid=Kn(),this.name="",this.type="BufferGeometry",this.index=null,this.attributes={},this.morphAttributes={},this.morphTargetsRelative=!1,this.groups=[],this.boundingBox=null,this.boundingSphere=null,this.drawRange={start:0,count:1/0},this.userData={}}getIndex(){return this.index}setIndex(e){return Array.isArray(e)?this.index=new(Nv(e)?kv:Ov)(e,1):this.index=e,this}getAttribute(e){return this.attributes[e]}setAttribute(e,t){return this.attributes[e]=t,this}deleteAttribute(e){return delete this.attributes[e],this}hasAttribute(e){return this.attributes[e]!==void 0}addGroup(e,t,i=0){this.groups.push({start:e,count:t,materialIndex:i})}clearGroups(){this.groups=[]}setDrawRange(e,t){this.drawRange.start=e,this.drawRange.count=t}applyMatrix4(e){const t=this.attributes.position;t!==void 0&&(t.applyMatrix4(e),t.needsUpdate=!0);const i=this.attributes.normal;if(i!==void 0){const s=new ze().getNormalMatrix(e);i.applyNormalMatrix(s),i.needsUpdate=!0}const r=this.attributes.tangent;return r!==void 0&&(r.transformDirection(e),r.needsUpdate=!0),this.boundingBox!==null&&this.computeBoundingBox(),this.boundingSphere!==null&&this.computeBoundingSphere(),this}applyQuaternion(e){return wn.makeRotationFromQuaternion(e),this.applyMatrix4(wn),this}rotateX(e){return wn.makeRotationX(e),this.applyMatrix4(wn),this}rotateY(e){return wn.makeRotationY(e),this.applyMatrix4(wn),this}rotateZ(e){return wn.makeRotationZ(e),this.applyMatrix4(wn),this}translate(e,t,i){return wn.makeTranslation(e,t,i),this.applyMatrix4(wn),this}scale(e,t,i){return wn.makeScale(e,t,i),this.applyMatrix4(wn),this}lookAt(e){return Ou.lookAt(e),Ou.updateMatrix(),this.applyMatrix4(Ou.matrix),this}center(){return this.computeBoundingBox(),this.boundingBox.getCenter(is).negate(),this.translate(is.x,is.y,is.z),this}setFromPoints(e){const t=[];for(let i=0,r=e.length;i<r;i++){const s=e[i];t.push(s.x,s.y,s.z||0)}return this.setAttribute("position",new In(t,3)),this}computeBoundingBox(){this.boundingBox===null&&(this.boundingBox=new Fi);const e=this.attributes.position,t=this.morphAttributes.position;if(e&&e.isGLBufferAttribute){console.error("THREE.BufferGeometry.computeBoundingBox(): GLBufferAttribute requires a manual bounding box.",this),this.boundingBox.set(new L(-1/0,-1/0,-1/0),new L(1/0,1/0,1/0));return}if(e!==void 0){if(this.boundingBox.setFromBufferAttribute(e),t)for(let i=0,r=t.length;i<r;i++){const s=t[i];_n.setFromBufferAttribute(s),this.morphTargetsRelative?(Nt.addVectors(this.boundingBox.min,_n.min),this.boundingBox.expandByPoint(Nt),Nt.addVectors(this.boundingBox.max,_n.max),this.boundingBox.expandByPoint(Nt)):(this.boundingBox.expandByPoint(_n.min),this.boundingBox.expandByPoint(_n.max))}}else this.boundingBox.makeEmpty();(isNaN(this.boundingBox.min.x)||isNaN(this.boundingBox.min.y)||isNaN(this.boundingBox.min.z))&&console.error('THREE.BufferGeometry.computeBoundingBox(): Computed min/max have NaN values. The "position" attribute is likely to have NaN values.',this)}computeBoundingSphere(){this.boundingSphere===null&&(this.boundingSphere=new li);const e=this.attributes.position,t=this.morphAttributes.position;if(e&&e.isGLBufferAttribute){console.error("THREE.BufferGeometry.computeBoundingSphere(): GLBufferAttribute requires a manual bounding sphere.",this),this.boundingSphere.set(new L,1/0);return}if(e){const i=this.boundingSphere.center;if(_n.setFromBufferAttribute(e),t)for(let s=0,o=t.length;s<o;s++){const a=t[s];Eo.setFromBufferAttribute(a),this.morphTargetsRelative?(Nt.addVectors(_n.min,Eo.min),_n.expandByPoint(Nt),Nt.addVectors(_n.max,Eo.max),_n.expandByPoint(Nt)):(_n.expandByPoint(Eo.min),_n.expandByPoint(Eo.max))}_n.getCenter(i);let r=0;for(let s=0,o=e.count;s<o;s++)Nt.fromBufferAttribute(e,s),r=Math.max(r,i.distanceToSquared(Nt));if(t)for(let s=0,o=t.length;s<o;s++){const a=t[s],l=this.morphTargetsRelative;for(let c=0,u=a.count;c<u;c++)Nt.fromBufferAttribute(a,c),l&&(is.fromBufferAttribute(e,c),Nt.add(is)),r=Math.max(r,i.distanceToSquared(Nt))}this.boundingSphere.radius=Math.sqrt(r),isNaN(this.boundingSphere.radius)&&console.error('THREE.BufferGeometry.computeBoundingSphere(): Computed radius is NaN. The "position" attribute is likely to have NaN values.',this)}}computeTangents(){const e=this.index,t=this.attributes;if(e===null||t.position===void 0||t.normal===void 0||t.uv===void 0){console.error("THREE.BufferGeometry: .computeTangents() failed. Missing required attributes (index, position, normal or uv)");return}const i=t.position,r=t.normal,s=t.uv;this.hasAttribute("tangent")===!1&&this.setAttribute("tangent",new Rt(new Float32Array(4*i.count),4));const o=this.getAttribute("tangent"),a=[],l=[];for(let b=0;b<i.count;b++)a[b]=new L,l[b]=new L;const c=new L,u=new L,d=new L,f=new we,p=new we,g=new we,y=new L,m=new L;function h(b,T,M){c.fromBufferAttribute(i,b),u.fromBufferAttribute(i,T),d.fromBufferAttribute(i,M),f.fromBufferAttribute(s,b),p.fromBufferAttribute(s,T),g.fromBufferAttribute(s,M),u.sub(c),d.sub(c),p.sub(f),g.sub(f);const I=1/(p.x*g.y-g.x*p.y);isFinite(I)&&(y.copy(u).multiplyScalar(g.y).addScaledVector(d,-p.y).multiplyScalar(I),m.copy(d).multiplyScalar(p.x).addScaledVector(u,-g.x).multiplyScalar(I),a[b].add(y),a[T].add(y),a[M].add(y),l[b].add(m),l[T].add(m),l[M].add(m))}let _=this.groups;_.length===0&&(_=[{start:0,count:e.count}]);for(let b=0,T=_.length;b<T;++b){const M=_[b],I=M.start,V=M.count;for(let B=I,j=I+V;B<j;B+=3)h(e.getX(B+0),e.getX(B+1),e.getX(B+2))}const v=new L,x=new L,R=new L,A=new L;function w(b){R.fromBufferAttribute(r,b),A.copy(R);const T=a[b];v.copy(T),v.sub(R.multiplyScalar(R.dot(T))).normalize(),x.crossVectors(A,T);const I=x.dot(l[b])<0?-1:1;o.setXYZW(b,v.x,v.y,v.z,I)}for(let b=0,T=_.length;b<T;++b){const M=_[b],I=M.start,V=M.count;for(let B=I,j=I+V;B<j;B+=3)w(e.getX(B+0)),w(e.getX(B+1)),w(e.getX(B+2))}}computeVertexNormals(){const e=this.index,t=this.getAttribute("position");if(t!==void 0){let i=this.getAttribute("normal");if(i===void 0)i=new Rt(new Float32Array(t.count*3),3),this.setAttribute("normal",i);else for(let f=0,p=i.count;f<p;f++)i.setXYZ(f,0,0,0);const r=new L,s=new L,o=new L,a=new L,l=new L,c=new L,u=new L,d=new L;if(e)for(let f=0,p=e.count;f<p;f+=3){const g=e.getX(f+0),y=e.getX(f+1),m=e.getX(f+2);r.fromBufferAttribute(t,g),s.fromBufferAttribute(t,y),o.fromBufferAttribute(t,m),u.subVectors(o,s),d.subVectors(r,s),u.cross(d),a.fromBufferAttribute(i,g),l.fromBufferAttribute(i,y),c.fromBufferAttribute(i,m),a.add(u),l.add(u),c.add(u),i.setXYZ(g,a.x,a.y,a.z),i.setXYZ(y,l.x,l.y,l.z),i.setXYZ(m,c.x,c.y,c.z)}else for(let f=0,p=t.count;f<p;f+=3)r.fromBufferAttribute(t,f+0),s.fromBufferAttribute(t,f+1),o.fromBufferAttribute(t,f+2),u.subVectors(o,s),d.subVectors(r,s),u.cross(d),i.setXYZ(f+0,u.x,u.y,u.z),i.setXYZ(f+1,u.x,u.y,u.z),i.setXYZ(f+2,u.x,u.y,u.z);this.normalizeNormals(),i.needsUpdate=!0}}normalizeNormals(){const e=this.attributes.normal;for(let t=0,i=e.count;t<i;t++)Nt.fromBufferAttribute(e,t),Nt.normalize(),e.setXYZ(t,Nt.x,Nt.y,Nt.z)}toNonIndexed(){function e(a,l){const c=a.array,u=a.itemSize,d=a.normalized,f=new c.constructor(l.length*u);let p=0,g=0;for(let y=0,m=l.length;y<m;y++){a.isInterleavedBufferAttribute?p=l[y]*a.data.stride+a.offset:p=l[y]*u;for(let h=0;h<u;h++)f[g++]=c[p++]}return new Rt(f,u,d)}if(this.index===null)return console.warn("THREE.BufferGeometry.toNonIndexed(): BufferGeometry is already non-indexed."),this;const t=new hn,i=this.index.array,r=this.attributes;for(const a in r){const l=r[a],c=e(l,i);t.setAttribute(a,c)}const s=this.morphAttributes;for(const a in s){const l=[],c=s[a];for(let u=0,d=c.length;u<d;u++){const f=c[u],p=e(f,i);l.push(p)}t.morphAttributes[a]=l}t.morphTargetsRelative=this.morphTargetsRelative;const o=this.groups;for(let a=0,l=o.length;a<l;a++){const c=o[a];t.addGroup(c.start,c.count,c.materialIndex)}return t}toJSON(){const e={metadata:{version:4.6,type:"BufferGeometry",generator:"BufferGeometry.toJSON"}};if(e.uuid=this.uuid,e.type=this.type,this.name!==""&&(e.name=this.name),Object.keys(this.userData).length>0&&(e.userData=this.userData),this.parameters!==void 0){const l=this.parameters;for(const c in l)l[c]!==void 0&&(e[c]=l[c]);return e}e.data={attributes:{}};const t=this.index;t!==null&&(e.data.index={type:t.array.constructor.name,array:Array.prototype.slice.call(t.array)});const i=this.attributes;for(const l in i){const c=i[l];e.data.attributes[l]=c.toJSON(e.data)}const r={};let s=!1;for(const l in this.morphAttributes){const c=this.morphAttributes[l],u=[];for(let d=0,f=c.length;d<f;d++){const p=c[d];u.push(p.toJSON(e.data))}u.length>0&&(r[l]=u,s=!0)}s&&(e.data.morphAttributes=r,e.data.morphTargetsRelative=this.morphTargetsRelative);const o=this.groups;o.length>0&&(e.data.groups=JSON.parse(JSON.stringify(o)));const a=this.boundingSphere;return a!==null&&(e.data.boundingSphere={center:a.center.toArray(),radius:a.radius}),e}clone(){return new this.constructor().copy(this)}copy(e){this.index=null,this.attributes={},this.morphAttributes={},this.groups=[],this.boundingBox=null,this.boundingSphere=null;const t={};this.name=e.name;const i=e.index;i!==null&&this.setIndex(i.clone(t));const r=e.attributes;for(const c in r){const u=r[c];this.setAttribute(c,u.clone(t))}const s=e.morphAttributes;for(const c in s){const u=[],d=s[c];for(let f=0,p=d.length;f<p;f++)u.push(d[f].clone(t));this.morphAttributes[c]=u}this.morphTargetsRelative=e.morphTargetsRelative;const o=e.groups;for(let c=0,u=o.length;c<u;c++){const d=o[c];this.addGroup(d.start,d.count,d.materialIndex)}const a=e.boundingBox;a!==null&&(this.boundingBox=a.clone());const l=e.boundingSphere;return l!==null&&(this.boundingSphere=l.clone()),this.drawRange.start=e.drawRange.start,this.drawRange.count=e.drawRange.count,this.userData=e.userData,this}dispose(){this.dispatchEvent({type:"dispose"})}}const Xm=new Ue,xr=new wa,nl=new li,jm=new L,rs=new L,ss=new L,os=new L,ku=new L,il=new L,rl=new we,sl=new we,ol=new we,Ym=new L,Km=new L,qm=new L,al=new L,ll=new L;class tn extends dt{constructor(e=new hn,t=new Ai){super(),this.isMesh=!0,this.type="Mesh",this.geometry=e,this.material=t,this.updateMorphTargets()}copy(e,t){return super.copy(e,t),e.morphTargetInfluences!==void 0&&(this.morphTargetInfluences=e.morphTargetInfluences.slice()),e.morphTargetDictionary!==void 0&&(this.morphTargetDictionary=Object.assign({},e.morphTargetDictionary)),this.material=Array.isArray(e.material)?e.material.slice():e.material,this.geometry=e.geometry,this}updateMorphTargets(){const t=this.geometry.morphAttributes,i=Object.keys(t);if(i.length>0){const r=t[i[0]];if(r!==void 0){this.morphTargetInfluences=[],this.morphTargetDictionary={};for(let s=0,o=r.length;s<o;s++){const a=r[s].name||String(s);this.morphTargetInfluences.push(0),this.morphTargetDictionary[a]=s}}}}getVertexPosition(e,t){const i=this.geometry,r=i.attributes.position,s=i.morphAttributes.position,o=i.morphTargetsRelative;t.fromBufferAttribute(r,e);const a=this.morphTargetInfluences;if(s&&a){il.set(0,0,0);for(let l=0,c=s.length;l<c;l++){const u=a[l],d=s[l];u!==0&&(ku.fromBufferAttribute(d,e),o?il.addScaledVector(ku,u):il.addScaledVector(ku.sub(t),u))}t.add(il)}return t}raycast(e,t){const i=this.geometry,r=this.material,s=this.matrixWorld;r!==void 0&&(i.boundingSphere===null&&i.computeBoundingSphere(),nl.copy(i.boundingSphere),nl.applyMatrix4(s),xr.copy(e.ray).recast(e.near),!(nl.containsPoint(xr.origin)===!1&&(xr.intersectSphere(nl,jm)===null||xr.origin.distanceToSquared(jm)>(e.far-e.near)**2))&&(Xm.copy(s).invert(),xr.copy(e.ray).applyMatrix4(Xm),!(i.boundingBox!==null&&xr.intersectsBox(i.boundingBox)===!1)&&this._computeIntersections(e,t,xr)))}_computeIntersections(e,t,i){let r;const s=this.geometry,o=this.material,a=s.index,l=s.attributes.position,c=s.attributes.uv,u=s.attributes.uv1,d=s.attributes.normal,f=s.groups,p=s.drawRange;if(a!==null)if(Array.isArray(o))for(let g=0,y=f.length;g<y;g++){const m=f[g],h=o[m.materialIndex],_=Math.max(m.start,p.start),v=Math.min(a.count,Math.min(m.start+m.count,p.start+p.count));for(let x=_,R=v;x<R;x+=3){const A=a.getX(x),w=a.getX(x+1),b=a.getX(x+2);r=cl(this,h,e,i,c,u,d,A,w,b),r&&(r.faceIndex=Math.floor(x/3),r.face.materialIndex=m.materialIndex,t.push(r))}}else{const g=Math.max(0,p.start),y=Math.min(a.count,p.start+p.count);for(let m=g,h=y;m<h;m+=3){const _=a.getX(m),v=a.getX(m+1),x=a.getX(m+2);r=cl(this,o,e,i,c,u,d,_,v,x),r&&(r.faceIndex=Math.floor(m/3),t.push(r))}}else if(l!==void 0)if(Array.isArray(o))for(let g=0,y=f.length;g<y;g++){const m=f[g],h=o[m.materialIndex],_=Math.max(m.start,p.start),v=Math.min(l.count,Math.min(m.start+m.count,p.start+p.count));for(let x=_,R=v;x<R;x+=3){const A=x,w=x+1,b=x+2;r=cl(this,h,e,i,c,u,d,A,w,b),r&&(r.faceIndex=Math.floor(x/3),r.face.materialIndex=m.materialIndex,t.push(r))}}else{const g=Math.max(0,p.start),y=Math.min(l.count,p.start+p.count);for(let m=g,h=y;m<h;m+=3){const _=m,v=m+1,x=m+2;r=cl(this,o,e,i,c,u,d,_,v,x),r&&(r.faceIndex=Math.floor(m/3),t.push(r))}}}}function CE(n,e,t,i,r,s,o,a){let l;if(e.side===fn?l=i.intersectTriangle(o,s,r,!0,a):l=i.intersectTriangle(r,s,o,e.side===Ii,a),l===null)return null;ll.copy(a),ll.applyMatrix4(n.matrixWorld);const c=t.ray.origin.distanceTo(ll);return c<t.near||c>t.far?null:{distance:c,point:ll.clone(),object:n}}function cl(n,e,t,i,r,s,o,a,l,c){n.getVertexPosition(a,rs),n.getVertexPosition(l,ss),n.getVertexPosition(c,os);const u=CE(n,e,t,i,rs,ss,os,al);if(u){r&&(rl.fromBufferAttribute(r,a),sl.fromBufferAttribute(r,l),ol.fromBufferAttribute(r,c),u.uv=Wn.getInterpolation(al,rs,ss,os,rl,sl,ol,new we)),s&&(rl.fromBufferAttribute(s,a),sl.fromBufferAttribute(s,l),ol.fromBufferAttribute(s,c),u.uv1=Wn.getInterpolation(al,rs,ss,os,rl,sl,ol,new we)),o&&(Ym.fromBufferAttribute(o,a),Km.fromBufferAttribute(o,l),qm.fromBufferAttribute(o,c),u.normal=Wn.getInterpolation(al,rs,ss,os,Ym,Km,qm,new L),u.normal.dot(i.direction)>0&&u.normal.multiplyScalar(-1));const d={a,b:l,c,normal:new L,materialIndex:0};Wn.getNormal(rs,ss,os,d.normal),u.face=d}return u}class Aa extends hn{constructor(e=1,t=1,i=1,r=1,s=1,o=1){super(),this.type="BoxGeometry",this.parameters={width:e,height:t,depth:i,widthSegments:r,heightSegments:s,depthSegments:o};const a=this;r=Math.floor(r),s=Math.floor(s),o=Math.floor(o);const l=[],c=[],u=[],d=[];let f=0,p=0;g("z","y","x",-1,-1,i,t,e,o,s,0),g("z","y","x",1,-1,i,t,-e,o,s,1),g("x","z","y",1,1,e,i,t,r,o,2),g("x","z","y",1,-1,e,i,-t,r,o,3),g("x","y","z",1,-1,e,t,i,r,s,4),g("x","y","z",-1,-1,e,t,-i,r,s,5),this.setIndex(l),this.setAttribute("position",new In(c,3)),this.setAttribute("normal",new In(u,3)),this.setAttribute("uv",new In(d,2));function g(y,m,h,_,v,x,R,A,w,b,T){const M=x/w,I=R/b,V=x/2,B=R/2,j=A/2,Y=w+1,W=b+1;let $=0,N=0;const K=new L;for(let Q=0;Q<W;Q++){const oe=Q*I-B;for(let Me=0;Me<Y;Me++){const je=Me*M-V;K[y]=je*_,K[m]=oe*v,K[h]=j,c.push(K.x,K.y,K.z),K[y]=0,K[m]=0,K[h]=A>0?1:-1,u.push(K.x,K.y,K.z),d.push(Me/w),d.push(1-Q/b),$+=1}}for(let Q=0;Q<b;Q++)for(let oe=0;oe<w;oe++){const Me=f+oe+Y*Q,je=f+oe+Y*(Q+1),G=f+(oe+1)+Y*(Q+1),ne=f+(oe+1)+Y*Q;l.push(Me,je,ne),l.push(je,G,ne),N+=6}a.addGroup(p,N,T),p+=N,f+=$}}copy(e){return super.copy(e),this.parameters=Object.assign({},e.parameters),this}static fromJSON(e){return new Aa(e.width,e.height,e.depth,e.widthSegments,e.heightSegments,e.depthSegments)}}function Qs(n){const e={};for(const t in n){e[t]={};for(const i in n[t]){const r=n[t][i];r&&(r.isColor||r.isMatrix3||r.isMatrix4||r.isVector2||r.isVector3||r.isVector4||r.isTexture||r.isQuaternion)?r.isRenderTargetTexture?(console.warn("UniformsUtils: Textures of render targets cannot be cloned via cloneUniforms() or mergeUniforms()."),e[t][i]=null):e[t][i]=r.clone():Array.isArray(r)?e[t][i]=r.slice():e[t][i]=r}}return e}function $t(n){const e={};for(let t=0;t<n.length;t++){const i=Qs(n[t]);for(const r in i)e[r]=i[r]}return e}function bE(n){const e=[];for(let t=0;t<n.length;t++)e.push(n[t].clone());return e}function Bv(n){const e=n.getRenderTarget();return e===null?n.outputColorSpace:e.isXRRenderTarget===!0?e.texture.colorSpace:qe.workingColorSpace}const PE={clone:Qs,merge:$t};var LE=`void main() {
	gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );
}`,IE=`void main() {
	gl_FragColor = vec4( 1.0, 0.0, 0.0, 1.0 );
}`;class Di extends qn{constructor(e){super(),this.isShaderMaterial=!0,this.type="ShaderMaterial",this.defines={},this.uniforms={},this.uniformsGroups=[],this.vertexShader=LE,this.fragmentShader=IE,this.linewidth=1,this.wireframe=!1,this.wireframeLinewidth=1,this.fog=!1,this.lights=!1,this.clipping=!1,this.forceSinglePass=!0,this.extensions={clipCullDistance:!1,multiDraw:!1},this.defaultAttributeValues={color:[1,1,1],uv:[0,0],uv1:[0,0]},this.index0AttributeName=void 0,this.uniformsNeedUpdate=!1,this.glslVersion=null,e!==void 0&&this.setValues(e)}copy(e){return super.copy(e),this.fragmentShader=e.fragmentShader,this.vertexShader=e.vertexShader,this.uniforms=Qs(e.uniforms),this.uniformsGroups=bE(e.uniformsGroups),this.defines=Object.assign({},e.defines),this.wireframe=e.wireframe,this.wireframeLinewidth=e.wireframeLinewidth,this.fog=e.fog,this.lights=e.lights,this.clipping=e.clipping,this.extensions=Object.assign({},e.extensions),this.glslVersion=e.glslVersion,this}toJSON(e){const t=super.toJSON(e);t.glslVersion=this.glslVersion,t.uniforms={};for(const r in this.uniforms){const o=this.uniforms[r].value;o&&o.isTexture?t.uniforms[r]={type:"t",value:o.toJSON(e).uuid}:o&&o.isColor?t.uniforms[r]={type:"c",value:o.getHex()}:o&&o.isVector2?t.uniforms[r]={type:"v2",value:o.toArray()}:o&&o.isVector3?t.uniforms[r]={type:"v3",value:o.toArray()}:o&&o.isVector4?t.uniforms[r]={type:"v4",value:o.toArray()}:o&&o.isMatrix3?t.uniforms[r]={type:"m3",value:o.toArray()}:o&&o.isMatrix4?t.uniforms[r]={type:"m4",value:o.toArray()}:t.uniforms[r]={value:o}}Object.keys(this.defines).length>0&&(t.defines=this.defines),t.vertexShader=this.vertexShader,t.fragmentShader=this.fragmentShader,t.lights=this.lights,t.clipping=this.clipping;const i={};for(const r in this.extensions)this.extensions[r]===!0&&(i[r]=!0);return Object.keys(i).length>0&&(t.extensions=i),t}}class zv extends dt{constructor(){super(),this.isCamera=!0,this.type="Camera",this.matrixWorldInverse=new Ue,this.projectionMatrix=new Ue,this.projectionMatrixInverse=new Ue,this.coordinateSystem=wi}copy(e,t){return super.copy(e,t),this.matrixWorldInverse.copy(e.matrixWorldInverse),this.projectionMatrix.copy(e.projectionMatrix),this.projectionMatrixInverse.copy(e.projectionMatrixInverse),this.coordinateSystem=e.coordinateSystem,this}getWorldDirection(e){return super.getWorldDirection(e).negate()}updateMatrixWorld(e){super.updateMatrixWorld(e),this.matrixWorldInverse.copy(this.matrixWorld).invert()}updateWorldMatrix(e,t){super.updateWorldMatrix(e,t),this.matrixWorldInverse.copy(this.matrixWorld).invert()}clone(){return new this.constructor().copy(this)}}const Wi=new L,$m=new we,Zm=new we;class Jt extends zv{constructor(e=50,t=1,i=.1,r=2e3){super(),this.isPerspectiveCamera=!0,this.type="PerspectiveCamera",this.fov=e,this.zoom=1,this.near=i,this.far=r,this.focus=10,this.aspect=t,this.view=null,this.filmGauge=35,this.filmOffset=0,this.updateProjectionMatrix()}copy(e,t){return super.copy(e,t),this.fov=e.fov,this.zoom=e.zoom,this.near=e.near,this.far=e.far,this.focus=e.focus,this.aspect=e.aspect,this.view=e.view===null?null:Object.assign({},e.view),this.filmGauge=e.filmGauge,this.filmOffset=e.filmOffset,this}setFocalLength(e){const t=.5*this.getFilmHeight()/e;this.fov=Zs*2*Math.atan(t),this.updateProjectionMatrix()}getFocalLength(){const e=Math.tan(Yo*.5*this.fov);return .5*this.getFilmHeight()/e}getEffectiveFOV(){return Zs*2*Math.atan(Math.tan(Yo*.5*this.fov)/this.zoom)}getFilmWidth(){return this.filmGauge*Math.min(this.aspect,1)}getFilmHeight(){return this.filmGauge/Math.max(this.aspect,1)}getViewBounds(e,t,i){Wi.set(-1,-1,.5).applyMatrix4(this.projectionMatrixInverse),t.set(Wi.x,Wi.y).multiplyScalar(-e/Wi.z),Wi.set(1,1,.5).applyMatrix4(this.projectionMatrixInverse),i.set(Wi.x,Wi.y).multiplyScalar(-e/Wi.z)}getViewSize(e,t){return this.getViewBounds(e,$m,Zm),t.subVectors(Zm,$m)}setViewOffset(e,t,i,r,s,o){this.aspect=e/t,this.view===null&&(this.view={enabled:!0,fullWidth:1,fullHeight:1,offsetX:0,offsetY:0,width:1,height:1}),this.view.enabled=!0,this.view.fullWidth=e,this.view.fullHeight=t,this.view.offsetX=i,this.view.offsetY=r,this.view.width=s,this.view.height=o,this.updateProjectionMatrix()}clearViewOffset(){this.view!==null&&(this.view.enabled=!1),this.updateProjectionMatrix()}updateProjectionMatrix(){const e=this.near;let t=e*Math.tan(Yo*.5*this.fov)/this.zoom,i=2*t,r=this.aspect*i,s=-.5*r;const o=this.view;if(this.view!==null&&this.view.enabled){const l=o.fullWidth,c=o.fullHeight;s+=o.offsetX*r/l,t-=o.offsetY*i/c,r*=o.width/l,i*=o.height/c}const a=this.filmOffset;a!==0&&(s+=e*a/this.getFilmWidth()),this.projectionMatrix.makePerspective(s,s+r,t,t-i,e,this.far,this.coordinateSystem),this.projectionMatrixInverse.copy(this.projectionMatrix).invert()}toJSON(e){const t=super.toJSON(e);return t.object.fov=this.fov,t.object.zoom=this.zoom,t.object.near=this.near,t.object.far=this.far,t.object.focus=this.focus,t.object.aspect=this.aspect,this.view!==null&&(t.object.view=Object.assign({},this.view)),t.object.filmGauge=this.filmGauge,t.object.filmOffset=this.filmOffset,t}}const as=-90,ls=1;class NE extends dt{constructor(e,t,i){super(),this.type="CubeCamera",this.renderTarget=i,this.coordinateSystem=null,this.activeMipmapLevel=0;const r=new Jt(as,ls,e,t);r.layers=this.layers,this.add(r);const s=new Jt(as,ls,e,t);s.layers=this.layers,this.add(s);const o=new Jt(as,ls,e,t);o.layers=this.layers,this.add(o);const a=new Jt(as,ls,e,t);a.layers=this.layers,this.add(a);const l=new Jt(as,ls,e,t);l.layers=this.layers,this.add(l);const c=new Jt(as,ls,e,t);c.layers=this.layers,this.add(c)}updateCoordinateSystem(){const e=this.coordinateSystem,t=this.children.concat(),[i,r,s,o,a,l]=t;for(const c of t)this.remove(c);if(e===wi)i.up.set(0,1,0),i.lookAt(1,0,0),r.up.set(0,1,0),r.lookAt(-1,0,0),s.up.set(0,0,-1),s.lookAt(0,1,0),o.up.set(0,0,1),o.lookAt(0,-1,0),a.up.set(0,1,0),a.lookAt(0,0,1),l.up.set(0,1,0),l.lookAt(0,0,-1);else if(e===Sc)i.up.set(0,-1,0),i.lookAt(-1,0,0),r.up.set(0,-1,0),r.lookAt(1,0,0),s.up.set(0,0,1),s.lookAt(0,1,0),o.up.set(0,0,-1),o.lookAt(0,-1,0),a.up.set(0,-1,0),a.lookAt(0,0,1),l.up.set(0,-1,0),l.lookAt(0,0,-1);else throw new Error("THREE.CubeCamera.updateCoordinateSystem(): Invalid coordinate system: "+e);for(const c of t)this.add(c),c.updateMatrixWorld()}update(e,t){this.parent===null&&this.updateMatrixWorld();const{renderTarget:i,activeMipmapLevel:r}=this;this.coordinateSystem!==e.coordinateSystem&&(this.coordinateSystem=e.coordinateSystem,this.updateCoordinateSystem());const[s,o,a,l,c,u]=this.children,d=e.getRenderTarget(),f=e.getActiveCubeFace(),p=e.getActiveMipmapLevel(),g=e.xr.enabled;e.xr.enabled=!1;const y=i.texture.generateMipmaps;i.texture.generateMipmaps=!1,e.setRenderTarget(i,0,r),e.render(t,s),e.setRenderTarget(i,1,r),e.render(t,o),e.setRenderTarget(i,2,r),e.render(t,a),e.setRenderTarget(i,3,r),e.render(t,l),e.setRenderTarget(i,4,r),e.render(t,c),i.texture.generateMipmaps=y,e.setRenderTarget(i,5,r),e.render(t,u),e.setRenderTarget(d,f,p),e.xr.enabled=g,i.texture.needsPMREMUpdate=!0}}class Hv extends Pt{constructor(e,t,i,r,s,o,a,l,c,u){e=e!==void 0?e:[],t=t!==void 0?t:js,super(e,t,i,r,s,o,a,l,c,u),this.isCubeTexture=!0,this.flipY=!1}get images(){return this.image}set images(e){this.image=e}}class DE extends Vr{constructor(e=1,t={}){super(e,e,t),this.isWebGLCubeRenderTarget=!0;const i={width:e,height:e,depth:1},r=[i,i,i,i,i,i];this.texture=new Hv(r,t.mapping,t.wrapS,t.wrapT,t.magFilter,t.minFilter,t.format,t.type,t.anisotropy,t.colorSpace),this.texture.isRenderTargetTexture=!0,this.texture.generateMipmaps=t.generateMipmaps!==void 0?t.generateMipmaps:!1,this.texture.minFilter=t.minFilter!==void 0?t.minFilter:an}fromEquirectangularTexture(e,t){this.texture.type=t.type,this.texture.colorSpace=t.colorSpace,this.texture.generateMipmaps=t.generateMipmaps,this.texture.minFilter=t.minFilter,this.texture.magFilter=t.magFilter;const i={uniforms:{tEquirect:{value:null}},vertexShader:`

				varying vec3 vWorldDirection;

				vec3 transformDirection( in vec3 dir, in mat4 matrix ) {

					return normalize( ( matrix * vec4( dir, 0.0 ) ).xyz );

				}

				void main() {

					vWorldDirection = transformDirection( position, modelMatrix );

					#include <begin_vertex>
					#include <project_vertex>

				}
			`,fragmentShader:`

				uniform sampler2D tEquirect;

				varying vec3 vWorldDirection;

				#include <common>

				void main() {

					vec3 direction = normalize( vWorldDirection );

					vec2 sampleUV = equirectUv( direction );

					gl_FragColor = texture2D( tEquirect, sampleUV );

				}
			`},r=new Aa(5,5,5),s=new Di({name:"CubemapFromEquirect",uniforms:Qs(i.uniforms),vertexShader:i.vertexShader,fragmentShader:i.fragmentShader,side:fn,blending:cr});s.uniforms.tEquirect.value=t;const o=new tn(r,s),a=t.minFilter;return t.minFilter===Ti&&(t.minFilter=an),new NE(1,10,this).update(e,o),t.minFilter=a,o.geometry.dispose(),o.material.dispose(),this}clear(e,t,i,r){const s=e.getRenderTarget();for(let o=0;o<6;o++)e.setRenderTarget(this,o),e.clear(t,i,r);e.setRenderTarget(s)}}const Bu=new L,UE=new L,FE=new ze;class Rr{constructor(e=new L(1,0,0),t=0){this.isPlane=!0,this.normal=e,this.constant=t}set(e,t){return this.normal.copy(e),this.constant=t,this}setComponents(e,t,i,r){return this.normal.set(e,t,i),this.constant=r,this}setFromNormalAndCoplanarPoint(e,t){return this.normal.copy(e),this.constant=-t.dot(this.normal),this}setFromCoplanarPoints(e,t,i){const r=Bu.subVectors(i,t).cross(UE.subVectors(e,t)).normalize();return this.setFromNormalAndCoplanarPoint(r,e),this}copy(e){return this.normal.copy(e.normal),this.constant=e.constant,this}normalize(){const e=1/this.normal.length();return this.normal.multiplyScalar(e),this.constant*=e,this}negate(){return this.constant*=-1,this.normal.negate(),this}distanceToPoint(e){return this.normal.dot(e)+this.constant}distanceToSphere(e){return this.distanceToPoint(e.center)-e.radius}projectPoint(e,t){return t.copy(e).addScaledVector(this.normal,-this.distanceToPoint(e))}intersectLine(e,t){const i=e.delta(Bu),r=this.normal.dot(i);if(r===0)return this.distanceToPoint(e.start)===0?t.copy(e.start):null;const s=-(e.start.dot(this.normal)+this.constant)/r;return s<0||s>1?null:t.copy(e.start).addScaledVector(i,s)}intersectsLine(e){const t=this.distanceToPoint(e.start),i=this.distanceToPoint(e.end);return t<0&&i>0||i<0&&t>0}intersectsBox(e){return e.intersectsPlane(this)}intersectsSphere(e){return e.intersectsPlane(this)}coplanarPoint(e){return e.copy(this.normal).multiplyScalar(-this.constant)}applyMatrix4(e,t){const i=t||FE.getNormalMatrix(e),r=this.coplanarPoint(Bu).applyMatrix4(e),s=this.normal.applyMatrix3(i).normalize();return this.constant=-r.dot(s),this}translate(e){return this.constant-=e.dot(this.normal),this}equals(e){return e.normal.equals(this.normal)&&e.constant===this.constant}clone(){return new this.constructor().copy(this)}}const Sr=new li,ul=new L;class Kh{constructor(e=new Rr,t=new Rr,i=new Rr,r=new Rr,s=new Rr,o=new Rr){this.planes=[e,t,i,r,s,o]}set(e,t,i,r,s,o){const a=this.planes;return a[0].copy(e),a[1].copy(t),a[2].copy(i),a[3].copy(r),a[4].copy(s),a[5].copy(o),this}copy(e){const t=this.planes;for(let i=0;i<6;i++)t[i].copy(e.planes[i]);return this}setFromProjectionMatrix(e,t=wi){const i=this.planes,r=e.elements,s=r[0],o=r[1],a=r[2],l=r[3],c=r[4],u=r[5],d=r[6],f=r[7],p=r[8],g=r[9],y=r[10],m=r[11],h=r[12],_=r[13],v=r[14],x=r[15];if(i[0].setComponents(l-s,f-c,m-p,x-h).normalize(),i[1].setComponents(l+s,f+c,m+p,x+h).normalize(),i[2].setComponents(l+o,f+u,m+g,x+_).normalize(),i[3].setComponents(l-o,f-u,m-g,x-_).normalize(),i[4].setComponents(l-a,f-d,m-y,x-v).normalize(),t===wi)i[5].setComponents(l+a,f+d,m+y,x+v).normalize();else if(t===Sc)i[5].setComponents(a,d,y,v).normalize();else throw new Error("THREE.Frustum.setFromProjectionMatrix(): Invalid coordinate system: "+t);return this}intersectsObject(e){if(e.boundingSphere!==void 0)e.boundingSphere===null&&e.computeBoundingSphere(),Sr.copy(e.boundingSphere).applyMatrix4(e.matrixWorld);else{const t=e.geometry;t.boundingSphere===null&&t.computeBoundingSphere(),Sr.copy(t.boundingSphere).applyMatrix4(e.matrixWorld)}return this.intersectsSphere(Sr)}intersectsSprite(e){return Sr.center.set(0,0,0),Sr.radius=.7071067811865476,Sr.applyMatrix4(e.matrixWorld),this.intersectsSphere(Sr)}intersectsSphere(e){const t=this.planes,i=e.center,r=-e.radius;for(let s=0;s<6;s++)if(t[s].distanceToPoint(i)<r)return!1;return!0}intersectsBox(e){const t=this.planes;for(let i=0;i<6;i++){const r=t[i];if(ul.x=r.normal.x>0?e.max.x:e.min.x,ul.y=r.normal.y>0?e.max.y:e.min.y,ul.z=r.normal.z>0?e.max.z:e.min.z,r.distanceToPoint(ul)<0)return!1}return!0}containsPoint(e){const t=this.planes;for(let i=0;i<6;i++)if(t[i].distanceToPoint(e)<0)return!1;return!0}clone(){return new this.constructor().copy(this)}}function Vv(){let n=null,e=!1,t=null,i=null;function r(s,o){t(s,o),i=n.requestAnimationFrame(r)}return{start:function(){e!==!0&&t!==null&&(i=n.requestAnimationFrame(r),e=!0)},stop:function(){n.cancelAnimationFrame(i),e=!1},setAnimationLoop:function(s){t=s},setContext:function(s){n=s}}}function OE(n){const e=new WeakMap;function t(a,l){const c=a.array,u=a.usage,d=c.byteLength,f=n.createBuffer();n.bindBuffer(l,f),n.bufferData(l,c,u),a.onUploadCallback();let p;if(c instanceof Float32Array)p=n.FLOAT;else if(c instanceof Uint16Array)a.isFloat16BufferAttribute?p=n.HALF_FLOAT:p=n.UNSIGNED_SHORT;else if(c instanceof Int16Array)p=n.SHORT;else if(c instanceof Uint32Array)p=n.UNSIGNED_INT;else if(c instanceof Int32Array)p=n.INT;else if(c instanceof Int8Array)p=n.BYTE;else if(c instanceof Uint8Array)p=n.UNSIGNED_BYTE;else if(c instanceof Uint8ClampedArray)p=n.UNSIGNED_BYTE;else throw new Error("THREE.WebGLAttributes: Unsupported buffer data format: "+c);return{buffer:f,type:p,bytesPerElement:c.BYTES_PER_ELEMENT,version:a.version,size:d}}function i(a,l,c){const u=l.array,d=l._updateRange,f=l.updateRanges;if(n.bindBuffer(c,a),d.count===-1&&f.length===0&&n.bufferSubData(c,0,u),f.length!==0){for(let p=0,g=f.length;p<g;p++){const y=f[p];n.bufferSubData(c,y.start*u.BYTES_PER_ELEMENT,u,y.start,y.count)}l.clearUpdateRanges()}d.count!==-1&&(n.bufferSubData(c,d.offset*u.BYTES_PER_ELEMENT,u,d.offset,d.count),d.count=-1),l.onUploadCallback()}function r(a){return a.isInterleavedBufferAttribute&&(a=a.data),e.get(a)}function s(a){a.isInterleavedBufferAttribute&&(a=a.data);const l=e.get(a);l&&(n.deleteBuffer(l.buffer),e.delete(a))}function o(a,l){if(a.isInterleavedBufferAttribute&&(a=a.data),a.isGLBufferAttribute){const u=e.get(a);(!u||u.version<a.version)&&e.set(a,{buffer:a.buffer,type:a.type,bytesPerElement:a.elementSize,version:a.version});return}const c=e.get(a);if(c===void 0)e.set(a,t(a,l));else if(c.version<a.version){if(c.size!==a.array.byteLength)throw new Error("THREE.WebGLAttributes: The size of the buffer attribute's array buffer does not match the original size. Resizing buffer attributes is not supported.");i(c.buffer,a,l),c.version=a.version}}return{get:r,remove:s,update:o}}class Vc extends hn{constructor(e=1,t=1,i=1,r=1){super(),this.type="PlaneGeometry",this.parameters={width:e,height:t,widthSegments:i,heightSegments:r};const s=e/2,o=t/2,a=Math.floor(i),l=Math.floor(r),c=a+1,u=l+1,d=e/a,f=t/l,p=[],g=[],y=[],m=[];for(let h=0;h<u;h++){const _=h*f-o;for(let v=0;v<c;v++){const x=v*d-s;g.push(x,-_,0),y.push(0,0,1),m.push(v/a),m.push(1-h/l)}}for(let h=0;h<l;h++)for(let _=0;_<a;_++){const v=_+c*h,x=_+c*(h+1),R=_+1+c*(h+1),A=_+1+c*h;p.push(v,x,A),p.push(x,R,A)}this.setIndex(p),this.setAttribute("position",new In(g,3)),this.setAttribute("normal",new In(y,3)),this.setAttribute("uv",new In(m,2))}copy(e){return super.copy(e),this.parameters=Object.assign({},e.parameters),this}static fromJSON(e){return new Vc(e.width,e.height,e.widthSegments,e.heightSegments)}}var kE=`#ifdef USE_ALPHAHASH
	if ( diffuseColor.a < getAlphaHashThreshold( vPosition ) ) discard;
#endif`,BE=`#ifdef USE_ALPHAHASH
	const float ALPHA_HASH_SCALE = 0.05;
	float hash2D( vec2 value ) {
		return fract( 1.0e4 * sin( 17.0 * value.x + 0.1 * value.y ) * ( 0.1 + abs( sin( 13.0 * value.y + value.x ) ) ) );
	}
	float hash3D( vec3 value ) {
		return hash2D( vec2( hash2D( value.xy ), value.z ) );
	}
	float getAlphaHashThreshold( vec3 position ) {
		float maxDeriv = max(
			length( dFdx( position.xyz ) ),
			length( dFdy( position.xyz ) )
		);
		float pixScale = 1.0 / ( ALPHA_HASH_SCALE * maxDeriv );
		vec2 pixScales = vec2(
			exp2( floor( log2( pixScale ) ) ),
			exp2( ceil( log2( pixScale ) ) )
		);
		vec2 alpha = vec2(
			hash3D( floor( pixScales.x * position.xyz ) ),
			hash3D( floor( pixScales.y * position.xyz ) )
		);
		float lerpFactor = fract( log2( pixScale ) );
		float x = ( 1.0 - lerpFactor ) * alpha.x + lerpFactor * alpha.y;
		float a = min( lerpFactor, 1.0 - lerpFactor );
		vec3 cases = vec3(
			x * x / ( 2.0 * a * ( 1.0 - a ) ),
			( x - 0.5 * a ) / ( 1.0 - a ),
			1.0 - ( ( 1.0 - x ) * ( 1.0 - x ) / ( 2.0 * a * ( 1.0 - a ) ) )
		);
		float threshold = ( x < ( 1.0 - a ) )
			? ( ( x < a ) ? cases.x : cases.y )
			: cases.z;
		return clamp( threshold , 1.0e-6, 1.0 );
	}
#endif`,zE=`#ifdef USE_ALPHAMAP
	diffuseColor.a *= texture2D( alphaMap, vAlphaMapUv ).g;
#endif`,HE=`#ifdef USE_ALPHAMAP
	uniform sampler2D alphaMap;
#endif`,VE=`#ifdef USE_ALPHATEST
	#ifdef ALPHA_TO_COVERAGE
	diffuseColor.a = smoothstep( alphaTest, alphaTest + fwidth( diffuseColor.a ), diffuseColor.a );
	if ( diffuseColor.a == 0.0 ) discard;
	#else
	if ( diffuseColor.a < alphaTest ) discard;
	#endif
#endif`,GE=`#ifdef USE_ALPHATEST
	uniform float alphaTest;
#endif`,WE=`#ifdef USE_AOMAP
	float ambientOcclusion = ( texture2D( aoMap, vAoMapUv ).r - 1.0 ) * aoMapIntensity + 1.0;
	reflectedLight.indirectDiffuse *= ambientOcclusion;
	#if defined( USE_CLEARCOAT ) 
		clearcoatSpecularIndirect *= ambientOcclusion;
	#endif
	#if defined( USE_SHEEN ) 
		sheenSpecularIndirect *= ambientOcclusion;
	#endif
	#if defined( USE_ENVMAP ) && defined( STANDARD )
		float dotNV = saturate( dot( geometryNormal, geometryViewDir ) );
		reflectedLight.indirectSpecular *= computeSpecularOcclusion( dotNV, ambientOcclusion, material.roughness );
	#endif
#endif`,XE=`#ifdef USE_AOMAP
	uniform sampler2D aoMap;
	uniform float aoMapIntensity;
#endif`,jE=`#ifdef USE_BATCHING
	#if ! defined( GL_ANGLE_multi_draw )
	#define gl_DrawID _gl_DrawID
	uniform int _gl_DrawID;
	#endif
	uniform highp sampler2D batchingTexture;
	uniform highp usampler2D batchingIdTexture;
	mat4 getBatchingMatrix( const in float i ) {
		int size = textureSize( batchingTexture, 0 ).x;
		int j = int( i ) * 4;
		int x = j % size;
		int y = j / size;
		vec4 v1 = texelFetch( batchingTexture, ivec2( x, y ), 0 );
		vec4 v2 = texelFetch( batchingTexture, ivec2( x + 1, y ), 0 );
		vec4 v3 = texelFetch( batchingTexture, ivec2( x + 2, y ), 0 );
		vec4 v4 = texelFetch( batchingTexture, ivec2( x + 3, y ), 0 );
		return mat4( v1, v2, v3, v4 );
	}
	float getIndirectIndex( const in int i ) {
		int size = textureSize( batchingIdTexture, 0 ).x;
		int x = i % size;
		int y = i / size;
		return float( texelFetch( batchingIdTexture, ivec2( x, y ), 0 ).r );
	}
#endif
#ifdef USE_BATCHING_COLOR
	uniform sampler2D batchingColorTexture;
	vec3 getBatchingColor( const in float i ) {
		int size = textureSize( batchingColorTexture, 0 ).x;
		int j = int( i );
		int x = j % size;
		int y = j / size;
		return texelFetch( batchingColorTexture, ivec2( x, y ), 0 ).rgb;
	}
#endif`,YE=`#ifdef USE_BATCHING
	mat4 batchingMatrix = getBatchingMatrix( getIndirectIndex( gl_DrawID ) );
#endif`,KE=`vec3 transformed = vec3( position );
#ifdef USE_ALPHAHASH
	vPosition = vec3( position );
#endif`,qE=`vec3 objectNormal = vec3( normal );
#ifdef USE_TANGENT
	vec3 objectTangent = vec3( tangent.xyz );
#endif`,$E=`float G_BlinnPhong_Implicit( ) {
	return 0.25;
}
float D_BlinnPhong( const in float shininess, const in float dotNH ) {
	return RECIPROCAL_PI * ( shininess * 0.5 + 1.0 ) * pow( dotNH, shininess );
}
vec3 BRDF_BlinnPhong( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in vec3 specularColor, const in float shininess ) {
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNH = saturate( dot( normal, halfDir ) );
	float dotVH = saturate( dot( viewDir, halfDir ) );
	vec3 F = F_Schlick( specularColor, 1.0, dotVH );
	float G = G_BlinnPhong_Implicit( );
	float D = D_BlinnPhong( shininess, dotNH );
	return F * ( G * D );
} // validated`,ZE=`#ifdef USE_IRIDESCENCE
	const mat3 XYZ_TO_REC709 = mat3(
		 3.2404542, -0.9692660,  0.0556434,
		-1.5371385,  1.8760108, -0.2040259,
		-0.4985314,  0.0415560,  1.0572252
	);
	vec3 Fresnel0ToIor( vec3 fresnel0 ) {
		vec3 sqrtF0 = sqrt( fresnel0 );
		return ( vec3( 1.0 ) + sqrtF0 ) / ( vec3( 1.0 ) - sqrtF0 );
	}
	vec3 IorToFresnel0( vec3 transmittedIor, float incidentIor ) {
		return pow2( ( transmittedIor - vec3( incidentIor ) ) / ( transmittedIor + vec3( incidentIor ) ) );
	}
	float IorToFresnel0( float transmittedIor, float incidentIor ) {
		return pow2( ( transmittedIor - incidentIor ) / ( transmittedIor + incidentIor ));
	}
	vec3 evalSensitivity( float OPD, vec3 shift ) {
		float phase = 2.0 * PI * OPD * 1.0e-9;
		vec3 val = vec3( 5.4856e-13, 4.4201e-13, 5.2481e-13 );
		vec3 pos = vec3( 1.6810e+06, 1.7953e+06, 2.2084e+06 );
		vec3 var = vec3( 4.3278e+09, 9.3046e+09, 6.6121e+09 );
		vec3 xyz = val * sqrt( 2.0 * PI * var ) * cos( pos * phase + shift ) * exp( - pow2( phase ) * var );
		xyz.x += 9.7470e-14 * sqrt( 2.0 * PI * 4.5282e+09 ) * cos( 2.2399e+06 * phase + shift[ 0 ] ) * exp( - 4.5282e+09 * pow2( phase ) );
		xyz /= 1.0685e-7;
		vec3 rgb = XYZ_TO_REC709 * xyz;
		return rgb;
	}
	vec3 evalIridescence( float outsideIOR, float eta2, float cosTheta1, float thinFilmThickness, vec3 baseF0 ) {
		vec3 I;
		float iridescenceIOR = mix( outsideIOR, eta2, smoothstep( 0.0, 0.03, thinFilmThickness ) );
		float sinTheta2Sq = pow2( outsideIOR / iridescenceIOR ) * ( 1.0 - pow2( cosTheta1 ) );
		float cosTheta2Sq = 1.0 - sinTheta2Sq;
		if ( cosTheta2Sq < 0.0 ) {
			return vec3( 1.0 );
		}
		float cosTheta2 = sqrt( cosTheta2Sq );
		float R0 = IorToFresnel0( iridescenceIOR, outsideIOR );
		float R12 = F_Schlick( R0, 1.0, cosTheta1 );
		float T121 = 1.0 - R12;
		float phi12 = 0.0;
		if ( iridescenceIOR < outsideIOR ) phi12 = PI;
		float phi21 = PI - phi12;
		vec3 baseIOR = Fresnel0ToIor( clamp( baseF0, 0.0, 0.9999 ) );		vec3 R1 = IorToFresnel0( baseIOR, iridescenceIOR );
		vec3 R23 = F_Schlick( R1, 1.0, cosTheta2 );
		vec3 phi23 = vec3( 0.0 );
		if ( baseIOR[ 0 ] < iridescenceIOR ) phi23[ 0 ] = PI;
		if ( baseIOR[ 1 ] < iridescenceIOR ) phi23[ 1 ] = PI;
		if ( baseIOR[ 2 ] < iridescenceIOR ) phi23[ 2 ] = PI;
		float OPD = 2.0 * iridescenceIOR * thinFilmThickness * cosTheta2;
		vec3 phi = vec3( phi21 ) + phi23;
		vec3 R123 = clamp( R12 * R23, 1e-5, 0.9999 );
		vec3 r123 = sqrt( R123 );
		vec3 Rs = pow2( T121 ) * R23 / ( vec3( 1.0 ) - R123 );
		vec3 C0 = R12 + Rs;
		I = C0;
		vec3 Cm = Rs - T121;
		for ( int m = 1; m <= 2; ++ m ) {
			Cm *= r123;
			vec3 Sm = 2.0 * evalSensitivity( float( m ) * OPD, float( m ) * phi );
			I += Cm * Sm;
		}
		return max( I, vec3( 0.0 ) );
	}
#endif`,QE=`#ifdef USE_BUMPMAP
	uniform sampler2D bumpMap;
	uniform float bumpScale;
	vec2 dHdxy_fwd() {
		vec2 dSTdx = dFdx( vBumpMapUv );
		vec2 dSTdy = dFdy( vBumpMapUv );
		float Hll = bumpScale * texture2D( bumpMap, vBumpMapUv ).x;
		float dBx = bumpScale * texture2D( bumpMap, vBumpMapUv + dSTdx ).x - Hll;
		float dBy = bumpScale * texture2D( bumpMap, vBumpMapUv + dSTdy ).x - Hll;
		return vec2( dBx, dBy );
	}
	vec3 perturbNormalArb( vec3 surf_pos, vec3 surf_norm, vec2 dHdxy, float faceDirection ) {
		vec3 vSigmaX = normalize( dFdx( surf_pos.xyz ) );
		vec3 vSigmaY = normalize( dFdy( surf_pos.xyz ) );
		vec3 vN = surf_norm;
		vec3 R1 = cross( vSigmaY, vN );
		vec3 R2 = cross( vN, vSigmaX );
		float fDet = dot( vSigmaX, R1 ) * faceDirection;
		vec3 vGrad = sign( fDet ) * ( dHdxy.x * R1 + dHdxy.y * R2 );
		return normalize( abs( fDet ) * surf_norm - vGrad );
	}
#endif`,JE=`#if NUM_CLIPPING_PLANES > 0
	vec4 plane;
	#ifdef ALPHA_TO_COVERAGE
		float distanceToPlane, distanceGradient;
		float clipOpacity = 1.0;
		#pragma unroll_loop_start
		for ( int i = 0; i < UNION_CLIPPING_PLANES; i ++ ) {
			plane = clippingPlanes[ i ];
			distanceToPlane = - dot( vClipPosition, plane.xyz ) + plane.w;
			distanceGradient = fwidth( distanceToPlane ) / 2.0;
			clipOpacity *= smoothstep( - distanceGradient, distanceGradient, distanceToPlane );
			if ( clipOpacity == 0.0 ) discard;
		}
		#pragma unroll_loop_end
		#if UNION_CLIPPING_PLANES < NUM_CLIPPING_PLANES
			float unionClipOpacity = 1.0;
			#pragma unroll_loop_start
			for ( int i = UNION_CLIPPING_PLANES; i < NUM_CLIPPING_PLANES; i ++ ) {
				plane = clippingPlanes[ i ];
				distanceToPlane = - dot( vClipPosition, plane.xyz ) + plane.w;
				distanceGradient = fwidth( distanceToPlane ) / 2.0;
				unionClipOpacity *= 1.0 - smoothstep( - distanceGradient, distanceGradient, distanceToPlane );
			}
			#pragma unroll_loop_end
			clipOpacity *= 1.0 - unionClipOpacity;
		#endif
		diffuseColor.a *= clipOpacity;
		if ( diffuseColor.a == 0.0 ) discard;
	#else
		#pragma unroll_loop_start
		for ( int i = 0; i < UNION_CLIPPING_PLANES; i ++ ) {
			plane = clippingPlanes[ i ];
			if ( dot( vClipPosition, plane.xyz ) > plane.w ) discard;
		}
		#pragma unroll_loop_end
		#if UNION_CLIPPING_PLANES < NUM_CLIPPING_PLANES
			bool clipped = true;
			#pragma unroll_loop_start
			for ( int i = UNION_CLIPPING_PLANES; i < NUM_CLIPPING_PLANES; i ++ ) {
				plane = clippingPlanes[ i ];
				clipped = ( dot( vClipPosition, plane.xyz ) > plane.w ) && clipped;
			}
			#pragma unroll_loop_end
			if ( clipped ) discard;
		#endif
	#endif
#endif`,eT=`#if NUM_CLIPPING_PLANES > 0
	varying vec3 vClipPosition;
	uniform vec4 clippingPlanes[ NUM_CLIPPING_PLANES ];
#endif`,tT=`#if NUM_CLIPPING_PLANES > 0
	varying vec3 vClipPosition;
#endif`,nT=`#if NUM_CLIPPING_PLANES > 0
	vClipPosition = - mvPosition.xyz;
#endif`,iT=`#if defined( USE_COLOR_ALPHA )
	diffuseColor *= vColor;
#elif defined( USE_COLOR )
	diffuseColor.rgb *= vColor;
#endif`,rT=`#if defined( USE_COLOR_ALPHA )
	varying vec4 vColor;
#elif defined( USE_COLOR )
	varying vec3 vColor;
#endif`,sT=`#if defined( USE_COLOR_ALPHA )
	varying vec4 vColor;
#elif defined( USE_COLOR ) || defined( USE_INSTANCING_COLOR ) || defined( USE_BATCHING_COLOR )
	varying vec3 vColor;
#endif`,oT=`#if defined( USE_COLOR_ALPHA )
	vColor = vec4( 1.0 );
#elif defined( USE_COLOR ) || defined( USE_INSTANCING_COLOR ) || defined( USE_BATCHING_COLOR )
	vColor = vec3( 1.0 );
#endif
#ifdef USE_COLOR
	vColor *= color;
#endif
#ifdef USE_INSTANCING_COLOR
	vColor.xyz *= instanceColor.xyz;
#endif
#ifdef USE_BATCHING_COLOR
	vec3 batchingColor = getBatchingColor( getIndirectIndex( gl_DrawID ) );
	vColor.xyz *= batchingColor.xyz;
#endif`,aT=`#define PI 3.141592653589793
#define PI2 6.283185307179586
#define PI_HALF 1.5707963267948966
#define RECIPROCAL_PI 0.3183098861837907
#define RECIPROCAL_PI2 0.15915494309189535
#define EPSILON 1e-6
#ifndef saturate
#define saturate( a ) clamp( a, 0.0, 1.0 )
#endif
#define whiteComplement( a ) ( 1.0 - saturate( a ) )
float pow2( const in float x ) { return x*x; }
vec3 pow2( const in vec3 x ) { return x*x; }
float pow3( const in float x ) { return x*x*x; }
float pow4( const in float x ) { float x2 = x*x; return x2*x2; }
float max3( const in vec3 v ) { return max( max( v.x, v.y ), v.z ); }
float average( const in vec3 v ) { return dot( v, vec3( 0.3333333 ) ); }
highp float rand( const in vec2 uv ) {
	const highp float a = 12.9898, b = 78.233, c = 43758.5453;
	highp float dt = dot( uv.xy, vec2( a,b ) ), sn = mod( dt, PI );
	return fract( sin( sn ) * c );
}
#ifdef HIGH_PRECISION
	float precisionSafeLength( vec3 v ) { return length( v ); }
#else
	float precisionSafeLength( vec3 v ) {
		float maxComponent = max3( abs( v ) );
		return length( v / maxComponent ) * maxComponent;
	}
#endif
struct IncidentLight {
	vec3 color;
	vec3 direction;
	bool visible;
};
struct ReflectedLight {
	vec3 directDiffuse;
	vec3 directSpecular;
	vec3 indirectDiffuse;
	vec3 indirectSpecular;
};
#ifdef USE_ALPHAHASH
	varying vec3 vPosition;
#endif
vec3 transformDirection( in vec3 dir, in mat4 matrix ) {
	return normalize( ( matrix * vec4( dir, 0.0 ) ).xyz );
}
vec3 inverseTransformDirection( in vec3 dir, in mat4 matrix ) {
	return normalize( ( vec4( dir, 0.0 ) * matrix ).xyz );
}
mat3 transposeMat3( const in mat3 m ) {
	mat3 tmp;
	tmp[ 0 ] = vec3( m[ 0 ].x, m[ 1 ].x, m[ 2 ].x );
	tmp[ 1 ] = vec3( m[ 0 ].y, m[ 1 ].y, m[ 2 ].y );
	tmp[ 2 ] = vec3( m[ 0 ].z, m[ 1 ].z, m[ 2 ].z );
	return tmp;
}
bool isPerspectiveMatrix( mat4 m ) {
	return m[ 2 ][ 3 ] == - 1.0;
}
vec2 equirectUv( in vec3 dir ) {
	float u = atan( dir.z, dir.x ) * RECIPROCAL_PI2 + 0.5;
	float v = asin( clamp( dir.y, - 1.0, 1.0 ) ) * RECIPROCAL_PI + 0.5;
	return vec2( u, v );
}
vec3 BRDF_Lambert( const in vec3 diffuseColor ) {
	return RECIPROCAL_PI * diffuseColor;
}
vec3 F_Schlick( const in vec3 f0, const in float f90, const in float dotVH ) {
	float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
	return f0 * ( 1.0 - fresnel ) + ( f90 * fresnel );
}
float F_Schlick( const in float f0, const in float f90, const in float dotVH ) {
	float fresnel = exp2( ( - 5.55473 * dotVH - 6.98316 ) * dotVH );
	return f0 * ( 1.0 - fresnel ) + ( f90 * fresnel );
} // validated`,lT=`#ifdef ENVMAP_TYPE_CUBE_UV
	#define cubeUV_minMipLevel 4.0
	#define cubeUV_minTileSize 16.0
	float getFace( vec3 direction ) {
		vec3 absDirection = abs( direction );
		float face = - 1.0;
		if ( absDirection.x > absDirection.z ) {
			if ( absDirection.x > absDirection.y )
				face = direction.x > 0.0 ? 0.0 : 3.0;
			else
				face = direction.y > 0.0 ? 1.0 : 4.0;
		} else {
			if ( absDirection.z > absDirection.y )
				face = direction.z > 0.0 ? 2.0 : 5.0;
			else
				face = direction.y > 0.0 ? 1.0 : 4.0;
		}
		return face;
	}
	vec2 getUV( vec3 direction, float face ) {
		vec2 uv;
		if ( face == 0.0 ) {
			uv = vec2( direction.z, direction.y ) / abs( direction.x );
		} else if ( face == 1.0 ) {
			uv = vec2( - direction.x, - direction.z ) / abs( direction.y );
		} else if ( face == 2.0 ) {
			uv = vec2( - direction.x, direction.y ) / abs( direction.z );
		} else if ( face == 3.0 ) {
			uv = vec2( - direction.z, direction.y ) / abs( direction.x );
		} else if ( face == 4.0 ) {
			uv = vec2( - direction.x, direction.z ) / abs( direction.y );
		} else {
			uv = vec2( direction.x, direction.y ) / abs( direction.z );
		}
		return 0.5 * ( uv + 1.0 );
	}
	vec3 bilinearCubeUV( sampler2D envMap, vec3 direction, float mipInt ) {
		float face = getFace( direction );
		float filterInt = max( cubeUV_minMipLevel - mipInt, 0.0 );
		mipInt = max( mipInt, cubeUV_minMipLevel );
		float faceSize = exp2( mipInt );
		highp vec2 uv = getUV( direction, face ) * ( faceSize - 2.0 ) + 1.0;
		if ( face > 2.0 ) {
			uv.y += faceSize;
			face -= 3.0;
		}
		uv.x += face * faceSize;
		uv.x += filterInt * 3.0 * cubeUV_minTileSize;
		uv.y += 4.0 * ( exp2( CUBEUV_MAX_MIP ) - faceSize );
		uv.x *= CUBEUV_TEXEL_WIDTH;
		uv.y *= CUBEUV_TEXEL_HEIGHT;
		#ifdef texture2DGradEXT
			return texture2DGradEXT( envMap, uv, vec2( 0.0 ), vec2( 0.0 ) ).rgb;
		#else
			return texture2D( envMap, uv ).rgb;
		#endif
	}
	#define cubeUV_r0 1.0
	#define cubeUV_m0 - 2.0
	#define cubeUV_r1 0.8
	#define cubeUV_m1 - 1.0
	#define cubeUV_r4 0.4
	#define cubeUV_m4 2.0
	#define cubeUV_r5 0.305
	#define cubeUV_m5 3.0
	#define cubeUV_r6 0.21
	#define cubeUV_m6 4.0
	float roughnessToMip( float roughness ) {
		float mip = 0.0;
		if ( roughness >= cubeUV_r1 ) {
			mip = ( cubeUV_r0 - roughness ) * ( cubeUV_m1 - cubeUV_m0 ) / ( cubeUV_r0 - cubeUV_r1 ) + cubeUV_m0;
		} else if ( roughness >= cubeUV_r4 ) {
			mip = ( cubeUV_r1 - roughness ) * ( cubeUV_m4 - cubeUV_m1 ) / ( cubeUV_r1 - cubeUV_r4 ) + cubeUV_m1;
		} else if ( roughness >= cubeUV_r5 ) {
			mip = ( cubeUV_r4 - roughness ) * ( cubeUV_m5 - cubeUV_m4 ) / ( cubeUV_r4 - cubeUV_r5 ) + cubeUV_m4;
		} else if ( roughness >= cubeUV_r6 ) {
			mip = ( cubeUV_r5 - roughness ) * ( cubeUV_m6 - cubeUV_m5 ) / ( cubeUV_r5 - cubeUV_r6 ) + cubeUV_m5;
		} else {
			mip = - 2.0 * log2( 1.16 * roughness );		}
		return mip;
	}
	vec4 textureCubeUV( sampler2D envMap, vec3 sampleDir, float roughness ) {
		float mip = clamp( roughnessToMip( roughness ), cubeUV_m0, CUBEUV_MAX_MIP );
		float mipF = fract( mip );
		float mipInt = floor( mip );
		vec3 color0 = bilinearCubeUV( envMap, sampleDir, mipInt );
		if ( mipF == 0.0 ) {
			return vec4( color0, 1.0 );
		} else {
			vec3 color1 = bilinearCubeUV( envMap, sampleDir, mipInt + 1.0 );
			return vec4( mix( color0, color1, mipF ), 1.0 );
		}
	}
#endif`,cT=`vec3 transformedNormal = objectNormal;
#ifdef USE_TANGENT
	vec3 transformedTangent = objectTangent;
#endif
#ifdef USE_BATCHING
	mat3 bm = mat3( batchingMatrix );
	transformedNormal /= vec3( dot( bm[ 0 ], bm[ 0 ] ), dot( bm[ 1 ], bm[ 1 ] ), dot( bm[ 2 ], bm[ 2 ] ) );
	transformedNormal = bm * transformedNormal;
	#ifdef USE_TANGENT
		transformedTangent = bm * transformedTangent;
	#endif
#endif
#ifdef USE_INSTANCING
	mat3 im = mat3( instanceMatrix );
	transformedNormal /= vec3( dot( im[ 0 ], im[ 0 ] ), dot( im[ 1 ], im[ 1 ] ), dot( im[ 2 ], im[ 2 ] ) );
	transformedNormal = im * transformedNormal;
	#ifdef USE_TANGENT
		transformedTangent = im * transformedTangent;
	#endif
#endif
transformedNormal = normalMatrix * transformedNormal;
#ifdef FLIP_SIDED
	transformedNormal = - transformedNormal;
#endif
#ifdef USE_TANGENT
	transformedTangent = ( modelViewMatrix * vec4( transformedTangent, 0.0 ) ).xyz;
	#ifdef FLIP_SIDED
		transformedTangent = - transformedTangent;
	#endif
#endif`,uT=`#ifdef USE_DISPLACEMENTMAP
	uniform sampler2D displacementMap;
	uniform float displacementScale;
	uniform float displacementBias;
#endif`,dT=`#ifdef USE_DISPLACEMENTMAP
	transformed += normalize( objectNormal ) * ( texture2D( displacementMap, vDisplacementMapUv ).x * displacementScale + displacementBias );
#endif`,fT=`#ifdef USE_EMISSIVEMAP
	vec4 emissiveColor = texture2D( emissiveMap, vEmissiveMapUv );
	totalEmissiveRadiance *= emissiveColor.rgb;
#endif`,hT=`#ifdef USE_EMISSIVEMAP
	uniform sampler2D emissiveMap;
#endif`,pT="gl_FragColor = linearToOutputTexel( gl_FragColor );",mT=`
const mat3 LINEAR_SRGB_TO_LINEAR_DISPLAY_P3 = mat3(
	vec3( 0.8224621, 0.177538, 0.0 ),
	vec3( 0.0331941, 0.9668058, 0.0 ),
	vec3( 0.0170827, 0.0723974, 0.9105199 )
);
const mat3 LINEAR_DISPLAY_P3_TO_LINEAR_SRGB = mat3(
	vec3( 1.2249401, - 0.2249404, 0.0 ),
	vec3( - 0.0420569, 1.0420571, 0.0 ),
	vec3( - 0.0196376, - 0.0786361, 1.0982735 )
);
vec4 LinearSRGBToLinearDisplayP3( in vec4 value ) {
	return vec4( value.rgb * LINEAR_SRGB_TO_LINEAR_DISPLAY_P3, value.a );
}
vec4 LinearDisplayP3ToLinearSRGB( in vec4 value ) {
	return vec4( value.rgb * LINEAR_DISPLAY_P3_TO_LINEAR_SRGB, value.a );
}
vec4 LinearTransferOETF( in vec4 value ) {
	return value;
}
vec4 sRGBTransferOETF( in vec4 value ) {
	return vec4( mix( pow( value.rgb, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 ), value.rgb * 12.92, vec3( lessThanEqual( value.rgb, vec3( 0.0031308 ) ) ) ), value.a );
}`,gT=`#ifdef USE_ENVMAP
	#ifdef ENV_WORLDPOS
		vec3 cameraToFrag;
		if ( isOrthographic ) {
			cameraToFrag = normalize( vec3( - viewMatrix[ 0 ][ 2 ], - viewMatrix[ 1 ][ 2 ], - viewMatrix[ 2 ][ 2 ] ) );
		} else {
			cameraToFrag = normalize( vWorldPosition - cameraPosition );
		}
		vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
		#ifdef ENVMAP_MODE_REFLECTION
			vec3 reflectVec = reflect( cameraToFrag, worldNormal );
		#else
			vec3 reflectVec = refract( cameraToFrag, worldNormal, refractionRatio );
		#endif
	#else
		vec3 reflectVec = vReflect;
	#endif
	#ifdef ENVMAP_TYPE_CUBE
		vec4 envColor = textureCube( envMap, envMapRotation * vec3( flipEnvMap * reflectVec.x, reflectVec.yz ) );
	#else
		vec4 envColor = vec4( 0.0 );
	#endif
	#ifdef ENVMAP_BLENDING_MULTIPLY
		outgoingLight = mix( outgoingLight, outgoingLight * envColor.xyz, specularStrength * reflectivity );
	#elif defined( ENVMAP_BLENDING_MIX )
		outgoingLight = mix( outgoingLight, envColor.xyz, specularStrength * reflectivity );
	#elif defined( ENVMAP_BLENDING_ADD )
		outgoingLight += envColor.xyz * specularStrength * reflectivity;
	#endif
#endif`,_T=`#ifdef USE_ENVMAP
	uniform float envMapIntensity;
	uniform float flipEnvMap;
	uniform mat3 envMapRotation;
	#ifdef ENVMAP_TYPE_CUBE
		uniform samplerCube envMap;
	#else
		uniform sampler2D envMap;
	#endif
	
#endif`,vT=`#ifdef USE_ENVMAP
	uniform float reflectivity;
	#if defined( USE_BUMPMAP ) || defined( USE_NORMALMAP ) || defined( PHONG ) || defined( LAMBERT )
		#define ENV_WORLDPOS
	#endif
	#ifdef ENV_WORLDPOS
		varying vec3 vWorldPosition;
		uniform float refractionRatio;
	#else
		varying vec3 vReflect;
	#endif
#endif`,yT=`#ifdef USE_ENVMAP
	#if defined( USE_BUMPMAP ) || defined( USE_NORMALMAP ) || defined( PHONG ) || defined( LAMBERT )
		#define ENV_WORLDPOS
	#endif
	#ifdef ENV_WORLDPOS
		
		varying vec3 vWorldPosition;
	#else
		varying vec3 vReflect;
		uniform float refractionRatio;
	#endif
#endif`,xT=`#ifdef USE_ENVMAP
	#ifdef ENV_WORLDPOS
		vWorldPosition = worldPosition.xyz;
	#else
		vec3 cameraToVertex;
		if ( isOrthographic ) {
			cameraToVertex = normalize( vec3( - viewMatrix[ 0 ][ 2 ], - viewMatrix[ 1 ][ 2 ], - viewMatrix[ 2 ][ 2 ] ) );
		} else {
			cameraToVertex = normalize( worldPosition.xyz - cameraPosition );
		}
		vec3 worldNormal = inverseTransformDirection( transformedNormal, viewMatrix );
		#ifdef ENVMAP_MODE_REFLECTION
			vReflect = reflect( cameraToVertex, worldNormal );
		#else
			vReflect = refract( cameraToVertex, worldNormal, refractionRatio );
		#endif
	#endif
#endif`,ST=`#ifdef USE_FOG
	vFogDepth = - mvPosition.z;
#endif`,MT=`#ifdef USE_FOG
	varying float vFogDepth;
#endif`,ET=`#ifdef USE_FOG
	#ifdef FOG_EXP2
		float fogFactor = 1.0 - exp( - fogDensity * fogDensity * vFogDepth * vFogDepth );
	#else
		float fogFactor = smoothstep( fogNear, fogFar, vFogDepth );
	#endif
	gl_FragColor.rgb = mix( gl_FragColor.rgb, fogColor, fogFactor );
#endif`,TT=`#ifdef USE_FOG
	uniform vec3 fogColor;
	varying float vFogDepth;
	#ifdef FOG_EXP2
		uniform float fogDensity;
	#else
		uniform float fogNear;
		uniform float fogFar;
	#endif
#endif`,wT=`#ifdef USE_GRADIENTMAP
	uniform sampler2D gradientMap;
#endif
vec3 getGradientIrradiance( vec3 normal, vec3 lightDirection ) {
	float dotNL = dot( normal, lightDirection );
	vec2 coord = vec2( dotNL * 0.5 + 0.5, 0.0 );
	#ifdef USE_GRADIENTMAP
		return vec3( texture2D( gradientMap, coord ).r );
	#else
		vec2 fw = fwidth( coord ) * 0.5;
		return mix( vec3( 0.7 ), vec3( 1.0 ), smoothstep( 0.7 - fw.x, 0.7 + fw.x, coord.x ) );
	#endif
}`,AT=`#ifdef USE_LIGHTMAP
	uniform sampler2D lightMap;
	uniform float lightMapIntensity;
#endif`,RT=`LambertMaterial material;
material.diffuseColor = diffuseColor.rgb;
material.specularStrength = specularStrength;`,CT=`varying vec3 vViewPosition;
struct LambertMaterial {
	vec3 diffuseColor;
	float specularStrength;
};
void RE_Direct_Lambert( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in LambertMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectDiffuse_Lambert( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in LambertMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_Lambert
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Lambert`,bT=`uniform bool receiveShadow;
uniform vec3 ambientLightColor;
#if defined( USE_LIGHT_PROBES )
	uniform vec3 lightProbe[ 9 ];
#endif
vec3 shGetIrradianceAt( in vec3 normal, in vec3 shCoefficients[ 9 ] ) {
	float x = normal.x, y = normal.y, z = normal.z;
	vec3 result = shCoefficients[ 0 ] * 0.886227;
	result += shCoefficients[ 1 ] * 2.0 * 0.511664 * y;
	result += shCoefficients[ 2 ] * 2.0 * 0.511664 * z;
	result += shCoefficients[ 3 ] * 2.0 * 0.511664 * x;
	result += shCoefficients[ 4 ] * 2.0 * 0.429043 * x * y;
	result += shCoefficients[ 5 ] * 2.0 * 0.429043 * y * z;
	result += shCoefficients[ 6 ] * ( 0.743125 * z * z - 0.247708 );
	result += shCoefficients[ 7 ] * 2.0 * 0.429043 * x * z;
	result += shCoefficients[ 8 ] * 0.429043 * ( x * x - y * y );
	return result;
}
vec3 getLightProbeIrradiance( const in vec3 lightProbe[ 9 ], const in vec3 normal ) {
	vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
	vec3 irradiance = shGetIrradianceAt( worldNormal, lightProbe );
	return irradiance;
}
vec3 getAmbientLightIrradiance( const in vec3 ambientLightColor ) {
	vec3 irradiance = ambientLightColor;
	return irradiance;
}
float getDistanceAttenuation( const in float lightDistance, const in float cutoffDistance, const in float decayExponent ) {
	float distanceFalloff = 1.0 / max( pow( lightDistance, decayExponent ), 0.01 );
	if ( cutoffDistance > 0.0 ) {
		distanceFalloff *= pow2( saturate( 1.0 - pow4( lightDistance / cutoffDistance ) ) );
	}
	return distanceFalloff;
}
float getSpotAttenuation( const in float coneCosine, const in float penumbraCosine, const in float angleCosine ) {
	return smoothstep( coneCosine, penumbraCosine, angleCosine );
}
#if NUM_DIR_LIGHTS > 0
	struct DirectionalLight {
		vec3 direction;
		vec3 color;
	};
	uniform DirectionalLight directionalLights[ NUM_DIR_LIGHTS ];
	void getDirectionalLightInfo( const in DirectionalLight directionalLight, out IncidentLight light ) {
		light.color = directionalLight.color;
		light.direction = directionalLight.direction;
		light.visible = true;
	}
#endif
#if NUM_POINT_LIGHTS > 0
	struct PointLight {
		vec3 position;
		vec3 color;
		float distance;
		float decay;
	};
	uniform PointLight pointLights[ NUM_POINT_LIGHTS ];
	void getPointLightInfo( const in PointLight pointLight, const in vec3 geometryPosition, out IncidentLight light ) {
		vec3 lVector = pointLight.position - geometryPosition;
		light.direction = normalize( lVector );
		float lightDistance = length( lVector );
		light.color = pointLight.color;
		light.color *= getDistanceAttenuation( lightDistance, pointLight.distance, pointLight.decay );
		light.visible = ( light.color != vec3( 0.0 ) );
	}
#endif
#if NUM_SPOT_LIGHTS > 0
	struct SpotLight {
		vec3 position;
		vec3 direction;
		vec3 color;
		float distance;
		float decay;
		float coneCos;
		float penumbraCos;
	};
	uniform SpotLight spotLights[ NUM_SPOT_LIGHTS ];
	void getSpotLightInfo( const in SpotLight spotLight, const in vec3 geometryPosition, out IncidentLight light ) {
		vec3 lVector = spotLight.position - geometryPosition;
		light.direction = normalize( lVector );
		float angleCos = dot( light.direction, spotLight.direction );
		float spotAttenuation = getSpotAttenuation( spotLight.coneCos, spotLight.penumbraCos, angleCos );
		if ( spotAttenuation > 0.0 ) {
			float lightDistance = length( lVector );
			light.color = spotLight.color * spotAttenuation;
			light.color *= getDistanceAttenuation( lightDistance, spotLight.distance, spotLight.decay );
			light.visible = ( light.color != vec3( 0.0 ) );
		} else {
			light.color = vec3( 0.0 );
			light.visible = false;
		}
	}
#endif
#if NUM_RECT_AREA_LIGHTS > 0
	struct RectAreaLight {
		vec3 color;
		vec3 position;
		vec3 halfWidth;
		vec3 halfHeight;
	};
	uniform sampler2D ltc_1;	uniform sampler2D ltc_2;
	uniform RectAreaLight rectAreaLights[ NUM_RECT_AREA_LIGHTS ];
#endif
#if NUM_HEMI_LIGHTS > 0
	struct HemisphereLight {
		vec3 direction;
		vec3 skyColor;
		vec3 groundColor;
	};
	uniform HemisphereLight hemisphereLights[ NUM_HEMI_LIGHTS ];
	vec3 getHemisphereLightIrradiance( const in HemisphereLight hemiLight, const in vec3 normal ) {
		float dotNL = dot( normal, hemiLight.direction );
		float hemiDiffuseWeight = 0.5 * dotNL + 0.5;
		vec3 irradiance = mix( hemiLight.groundColor, hemiLight.skyColor, hemiDiffuseWeight );
		return irradiance;
	}
#endif`,PT=`#ifdef USE_ENVMAP
	vec3 getIBLIrradiance( const in vec3 normal ) {
		#ifdef ENVMAP_TYPE_CUBE_UV
			vec3 worldNormal = inverseTransformDirection( normal, viewMatrix );
			vec4 envMapColor = textureCubeUV( envMap, envMapRotation * worldNormal, 1.0 );
			return PI * envMapColor.rgb * envMapIntensity;
		#else
			return vec3( 0.0 );
		#endif
	}
	vec3 getIBLRadiance( const in vec3 viewDir, const in vec3 normal, const in float roughness ) {
		#ifdef ENVMAP_TYPE_CUBE_UV
			vec3 reflectVec = reflect( - viewDir, normal );
			reflectVec = normalize( mix( reflectVec, normal, roughness * roughness) );
			reflectVec = inverseTransformDirection( reflectVec, viewMatrix );
			vec4 envMapColor = textureCubeUV( envMap, envMapRotation * reflectVec, roughness );
			return envMapColor.rgb * envMapIntensity;
		#else
			return vec3( 0.0 );
		#endif
	}
	#ifdef USE_ANISOTROPY
		vec3 getIBLAnisotropyRadiance( const in vec3 viewDir, const in vec3 normal, const in float roughness, const in vec3 bitangent, const in float anisotropy ) {
			#ifdef ENVMAP_TYPE_CUBE_UV
				vec3 bentNormal = cross( bitangent, viewDir );
				bentNormal = normalize( cross( bentNormal, bitangent ) );
				bentNormal = normalize( mix( bentNormal, normal, pow2( pow2( 1.0 - anisotropy * ( 1.0 - roughness ) ) ) ) );
				return getIBLRadiance( viewDir, bentNormal, roughness );
			#else
				return vec3( 0.0 );
			#endif
		}
	#endif
#endif`,LT=`ToonMaterial material;
material.diffuseColor = diffuseColor.rgb;`,IT=`varying vec3 vViewPosition;
struct ToonMaterial {
	vec3 diffuseColor;
};
void RE_Direct_Toon( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in ToonMaterial material, inout ReflectedLight reflectedLight ) {
	vec3 irradiance = getGradientIrradiance( geometryNormal, directLight.direction ) * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectDiffuse_Toon( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in ToonMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_Toon
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Toon`,NT=`BlinnPhongMaterial material;
material.diffuseColor = diffuseColor.rgb;
material.specularColor = specular;
material.specularShininess = shininess;
material.specularStrength = specularStrength;`,DT=`varying vec3 vViewPosition;
struct BlinnPhongMaterial {
	vec3 diffuseColor;
	vec3 specularColor;
	float specularShininess;
	float specularStrength;
};
void RE_Direct_BlinnPhong( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in BlinnPhongMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
	reflectedLight.directSpecular += irradiance * BRDF_BlinnPhong( directLight.direction, geometryViewDir, geometryNormal, material.specularColor, material.specularShininess ) * material.specularStrength;
}
void RE_IndirectDiffuse_BlinnPhong( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in BlinnPhongMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
#define RE_Direct				RE_Direct_BlinnPhong
#define RE_IndirectDiffuse		RE_IndirectDiffuse_BlinnPhong`,UT=`PhysicalMaterial material;
material.diffuseColor = diffuseColor.rgb * ( 1.0 - metalnessFactor );
vec3 dxy = max( abs( dFdx( nonPerturbedNormal ) ), abs( dFdy( nonPerturbedNormal ) ) );
float geometryRoughness = max( max( dxy.x, dxy.y ), dxy.z );
material.roughness = max( roughnessFactor, 0.0525 );material.roughness += geometryRoughness;
material.roughness = min( material.roughness, 1.0 );
#ifdef IOR
	material.ior = ior;
	#ifdef USE_SPECULAR
		float specularIntensityFactor = specularIntensity;
		vec3 specularColorFactor = specularColor;
		#ifdef USE_SPECULAR_COLORMAP
			specularColorFactor *= texture2D( specularColorMap, vSpecularColorMapUv ).rgb;
		#endif
		#ifdef USE_SPECULAR_INTENSITYMAP
			specularIntensityFactor *= texture2D( specularIntensityMap, vSpecularIntensityMapUv ).a;
		#endif
		material.specularF90 = mix( specularIntensityFactor, 1.0, metalnessFactor );
	#else
		float specularIntensityFactor = 1.0;
		vec3 specularColorFactor = vec3( 1.0 );
		material.specularF90 = 1.0;
	#endif
	material.specularColor = mix( min( pow2( ( material.ior - 1.0 ) / ( material.ior + 1.0 ) ) * specularColorFactor, vec3( 1.0 ) ) * specularIntensityFactor, diffuseColor.rgb, metalnessFactor );
#else
	material.specularColor = mix( vec3( 0.04 ), diffuseColor.rgb, metalnessFactor );
	material.specularF90 = 1.0;
#endif
#ifdef USE_CLEARCOAT
	material.clearcoat = clearcoat;
	material.clearcoatRoughness = clearcoatRoughness;
	material.clearcoatF0 = vec3( 0.04 );
	material.clearcoatF90 = 1.0;
	#ifdef USE_CLEARCOATMAP
		material.clearcoat *= texture2D( clearcoatMap, vClearcoatMapUv ).x;
	#endif
	#ifdef USE_CLEARCOAT_ROUGHNESSMAP
		material.clearcoatRoughness *= texture2D( clearcoatRoughnessMap, vClearcoatRoughnessMapUv ).y;
	#endif
	material.clearcoat = saturate( material.clearcoat );	material.clearcoatRoughness = max( material.clearcoatRoughness, 0.0525 );
	material.clearcoatRoughness += geometryRoughness;
	material.clearcoatRoughness = min( material.clearcoatRoughness, 1.0 );
#endif
#ifdef USE_DISPERSION
	material.dispersion = dispersion;
#endif
#ifdef USE_IRIDESCENCE
	material.iridescence = iridescence;
	material.iridescenceIOR = iridescenceIOR;
	#ifdef USE_IRIDESCENCEMAP
		material.iridescence *= texture2D( iridescenceMap, vIridescenceMapUv ).r;
	#endif
	#ifdef USE_IRIDESCENCE_THICKNESSMAP
		material.iridescenceThickness = (iridescenceThicknessMaximum - iridescenceThicknessMinimum) * texture2D( iridescenceThicknessMap, vIridescenceThicknessMapUv ).g + iridescenceThicknessMinimum;
	#else
		material.iridescenceThickness = iridescenceThicknessMaximum;
	#endif
#endif
#ifdef USE_SHEEN
	material.sheenColor = sheenColor;
	#ifdef USE_SHEEN_COLORMAP
		material.sheenColor *= texture2D( sheenColorMap, vSheenColorMapUv ).rgb;
	#endif
	material.sheenRoughness = clamp( sheenRoughness, 0.07, 1.0 );
	#ifdef USE_SHEEN_ROUGHNESSMAP
		material.sheenRoughness *= texture2D( sheenRoughnessMap, vSheenRoughnessMapUv ).a;
	#endif
#endif
#ifdef USE_ANISOTROPY
	#ifdef USE_ANISOTROPYMAP
		mat2 anisotropyMat = mat2( anisotropyVector.x, anisotropyVector.y, - anisotropyVector.y, anisotropyVector.x );
		vec3 anisotropyPolar = texture2D( anisotropyMap, vAnisotropyMapUv ).rgb;
		vec2 anisotropyV = anisotropyMat * normalize( 2.0 * anisotropyPolar.rg - vec2( 1.0 ) ) * anisotropyPolar.b;
	#else
		vec2 anisotropyV = anisotropyVector;
	#endif
	material.anisotropy = length( anisotropyV );
	if( material.anisotropy == 0.0 ) {
		anisotropyV = vec2( 1.0, 0.0 );
	} else {
		anisotropyV /= material.anisotropy;
		material.anisotropy = saturate( material.anisotropy );
	}
	material.alphaT = mix( pow2( material.roughness ), 1.0, pow2( material.anisotropy ) );
	material.anisotropyT = tbn[ 0 ] * anisotropyV.x + tbn[ 1 ] * anisotropyV.y;
	material.anisotropyB = tbn[ 1 ] * anisotropyV.x - tbn[ 0 ] * anisotropyV.y;
#endif`,FT=`struct PhysicalMaterial {
	vec3 diffuseColor;
	float roughness;
	vec3 specularColor;
	float specularF90;
	float dispersion;
	#ifdef USE_CLEARCOAT
		float clearcoat;
		float clearcoatRoughness;
		vec3 clearcoatF0;
		float clearcoatF90;
	#endif
	#ifdef USE_IRIDESCENCE
		float iridescence;
		float iridescenceIOR;
		float iridescenceThickness;
		vec3 iridescenceFresnel;
		vec3 iridescenceF0;
	#endif
	#ifdef USE_SHEEN
		vec3 sheenColor;
		float sheenRoughness;
	#endif
	#ifdef IOR
		float ior;
	#endif
	#ifdef USE_TRANSMISSION
		float transmission;
		float transmissionAlpha;
		float thickness;
		float attenuationDistance;
		vec3 attenuationColor;
	#endif
	#ifdef USE_ANISOTROPY
		float anisotropy;
		float alphaT;
		vec3 anisotropyT;
		vec3 anisotropyB;
	#endif
};
vec3 clearcoatSpecularDirect = vec3( 0.0 );
vec3 clearcoatSpecularIndirect = vec3( 0.0 );
vec3 sheenSpecularDirect = vec3( 0.0 );
vec3 sheenSpecularIndirect = vec3(0.0 );
vec3 Schlick_to_F0( const in vec3 f, const in float f90, const in float dotVH ) {
    float x = clamp( 1.0 - dotVH, 0.0, 1.0 );
    float x2 = x * x;
    float x5 = clamp( x * x2 * x2, 0.0, 0.9999 );
    return ( f - vec3( f90 ) * x5 ) / ( 1.0 - x5 );
}
float V_GGX_SmithCorrelated( const in float alpha, const in float dotNL, const in float dotNV ) {
	float a2 = pow2( alpha );
	float gv = dotNL * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNV ) );
	float gl = dotNV * sqrt( a2 + ( 1.0 - a2 ) * pow2( dotNL ) );
	return 0.5 / max( gv + gl, EPSILON );
}
float D_GGX( const in float alpha, const in float dotNH ) {
	float a2 = pow2( alpha );
	float denom = pow2( dotNH ) * ( a2 - 1.0 ) + 1.0;
	return RECIPROCAL_PI * a2 / pow2( denom );
}
#ifdef USE_ANISOTROPY
	float V_GGX_SmithCorrelated_Anisotropic( const in float alphaT, const in float alphaB, const in float dotTV, const in float dotBV, const in float dotTL, const in float dotBL, const in float dotNV, const in float dotNL ) {
		float gv = dotNL * length( vec3( alphaT * dotTV, alphaB * dotBV, dotNV ) );
		float gl = dotNV * length( vec3( alphaT * dotTL, alphaB * dotBL, dotNL ) );
		float v = 0.5 / ( gv + gl );
		return saturate(v);
	}
	float D_GGX_Anisotropic( const in float alphaT, const in float alphaB, const in float dotNH, const in float dotTH, const in float dotBH ) {
		float a2 = alphaT * alphaB;
		highp vec3 v = vec3( alphaB * dotTH, alphaT * dotBH, a2 * dotNH );
		highp float v2 = dot( v, v );
		float w2 = a2 / v2;
		return RECIPROCAL_PI * a2 * pow2 ( w2 );
	}
#endif
#ifdef USE_CLEARCOAT
	vec3 BRDF_GGX_Clearcoat( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material) {
		vec3 f0 = material.clearcoatF0;
		float f90 = material.clearcoatF90;
		float roughness = material.clearcoatRoughness;
		float alpha = pow2( roughness );
		vec3 halfDir = normalize( lightDir + viewDir );
		float dotNL = saturate( dot( normal, lightDir ) );
		float dotNV = saturate( dot( normal, viewDir ) );
		float dotNH = saturate( dot( normal, halfDir ) );
		float dotVH = saturate( dot( viewDir, halfDir ) );
		vec3 F = F_Schlick( f0, f90, dotVH );
		float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
		float D = D_GGX( alpha, dotNH );
		return F * ( V * D );
	}
#endif
vec3 BRDF_GGX( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, const in PhysicalMaterial material ) {
	vec3 f0 = material.specularColor;
	float f90 = material.specularF90;
	float roughness = material.roughness;
	float alpha = pow2( roughness );
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNL = saturate( dot( normal, lightDir ) );
	float dotNV = saturate( dot( normal, viewDir ) );
	float dotNH = saturate( dot( normal, halfDir ) );
	float dotVH = saturate( dot( viewDir, halfDir ) );
	vec3 F = F_Schlick( f0, f90, dotVH );
	#ifdef USE_IRIDESCENCE
		F = mix( F, material.iridescenceFresnel, material.iridescence );
	#endif
	#ifdef USE_ANISOTROPY
		float dotTL = dot( material.anisotropyT, lightDir );
		float dotTV = dot( material.anisotropyT, viewDir );
		float dotTH = dot( material.anisotropyT, halfDir );
		float dotBL = dot( material.anisotropyB, lightDir );
		float dotBV = dot( material.anisotropyB, viewDir );
		float dotBH = dot( material.anisotropyB, halfDir );
		float V = V_GGX_SmithCorrelated_Anisotropic( material.alphaT, alpha, dotTV, dotBV, dotTL, dotBL, dotNV, dotNL );
		float D = D_GGX_Anisotropic( material.alphaT, alpha, dotNH, dotTH, dotBH );
	#else
		float V = V_GGX_SmithCorrelated( alpha, dotNL, dotNV );
		float D = D_GGX( alpha, dotNH );
	#endif
	return F * ( V * D );
}
vec2 LTC_Uv( const in vec3 N, const in vec3 V, const in float roughness ) {
	const float LUT_SIZE = 64.0;
	const float LUT_SCALE = ( LUT_SIZE - 1.0 ) / LUT_SIZE;
	const float LUT_BIAS = 0.5 / LUT_SIZE;
	float dotNV = saturate( dot( N, V ) );
	vec2 uv = vec2( roughness, sqrt( 1.0 - dotNV ) );
	uv = uv * LUT_SCALE + LUT_BIAS;
	return uv;
}
float LTC_ClippedSphereFormFactor( const in vec3 f ) {
	float l = length( f );
	return max( ( l * l + f.z ) / ( l + 1.0 ), 0.0 );
}
vec3 LTC_EdgeVectorFormFactor( const in vec3 v1, const in vec3 v2 ) {
	float x = dot( v1, v2 );
	float y = abs( x );
	float a = 0.8543985 + ( 0.4965155 + 0.0145206 * y ) * y;
	float b = 3.4175940 + ( 4.1616724 + y ) * y;
	float v = a / b;
	float theta_sintheta = ( x > 0.0 ) ? v : 0.5 * inversesqrt( max( 1.0 - x * x, 1e-7 ) ) - v;
	return cross( v1, v2 ) * theta_sintheta;
}
vec3 LTC_Evaluate( const in vec3 N, const in vec3 V, const in vec3 P, const in mat3 mInv, const in vec3 rectCoords[ 4 ] ) {
	vec3 v1 = rectCoords[ 1 ] - rectCoords[ 0 ];
	vec3 v2 = rectCoords[ 3 ] - rectCoords[ 0 ];
	vec3 lightNormal = cross( v1, v2 );
	if( dot( lightNormal, P - rectCoords[ 0 ] ) < 0.0 ) return vec3( 0.0 );
	vec3 T1, T2;
	T1 = normalize( V - N * dot( V, N ) );
	T2 = - cross( N, T1 );
	mat3 mat = mInv * transposeMat3( mat3( T1, T2, N ) );
	vec3 coords[ 4 ];
	coords[ 0 ] = mat * ( rectCoords[ 0 ] - P );
	coords[ 1 ] = mat * ( rectCoords[ 1 ] - P );
	coords[ 2 ] = mat * ( rectCoords[ 2 ] - P );
	coords[ 3 ] = mat * ( rectCoords[ 3 ] - P );
	coords[ 0 ] = normalize( coords[ 0 ] );
	coords[ 1 ] = normalize( coords[ 1 ] );
	coords[ 2 ] = normalize( coords[ 2 ] );
	coords[ 3 ] = normalize( coords[ 3 ] );
	vec3 vectorFormFactor = vec3( 0.0 );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 0 ], coords[ 1 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 1 ], coords[ 2 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 2 ], coords[ 3 ] );
	vectorFormFactor += LTC_EdgeVectorFormFactor( coords[ 3 ], coords[ 0 ] );
	float result = LTC_ClippedSphereFormFactor( vectorFormFactor );
	return vec3( result );
}
#if defined( USE_SHEEN )
float D_Charlie( float roughness, float dotNH ) {
	float alpha = pow2( roughness );
	float invAlpha = 1.0 / alpha;
	float cos2h = dotNH * dotNH;
	float sin2h = max( 1.0 - cos2h, 0.0078125 );
	return ( 2.0 + invAlpha ) * pow( sin2h, invAlpha * 0.5 ) / ( 2.0 * PI );
}
float V_Neubelt( float dotNV, float dotNL ) {
	return saturate( 1.0 / ( 4.0 * ( dotNL + dotNV - dotNL * dotNV ) ) );
}
vec3 BRDF_Sheen( const in vec3 lightDir, const in vec3 viewDir, const in vec3 normal, vec3 sheenColor, const in float sheenRoughness ) {
	vec3 halfDir = normalize( lightDir + viewDir );
	float dotNL = saturate( dot( normal, lightDir ) );
	float dotNV = saturate( dot( normal, viewDir ) );
	float dotNH = saturate( dot( normal, halfDir ) );
	float D = D_Charlie( sheenRoughness, dotNH );
	float V = V_Neubelt( dotNV, dotNL );
	return sheenColor * ( D * V );
}
#endif
float IBLSheenBRDF( const in vec3 normal, const in vec3 viewDir, const in float roughness ) {
	float dotNV = saturate( dot( normal, viewDir ) );
	float r2 = roughness * roughness;
	float a = roughness < 0.25 ? -339.2 * r2 + 161.4 * roughness - 25.9 : -8.48 * r2 + 14.3 * roughness - 9.95;
	float b = roughness < 0.25 ? 44.0 * r2 - 23.7 * roughness + 3.26 : 1.97 * r2 - 3.27 * roughness + 0.72;
	float DG = exp( a * dotNV + b ) + ( roughness < 0.25 ? 0.0 : 0.1 * ( roughness - 0.25 ) );
	return saturate( DG * RECIPROCAL_PI );
}
vec2 DFGApprox( const in vec3 normal, const in vec3 viewDir, const in float roughness ) {
	float dotNV = saturate( dot( normal, viewDir ) );
	const vec4 c0 = vec4( - 1, - 0.0275, - 0.572, 0.022 );
	const vec4 c1 = vec4( 1, 0.0425, 1.04, - 0.04 );
	vec4 r = roughness * c0 + c1;
	float a004 = min( r.x * r.x, exp2( - 9.28 * dotNV ) ) * r.x + r.y;
	vec2 fab = vec2( - 1.04, 1.04 ) * a004 + r.zw;
	return fab;
}
vec3 EnvironmentBRDF( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float roughness ) {
	vec2 fab = DFGApprox( normal, viewDir, roughness );
	return specularColor * fab.x + specularF90 * fab.y;
}
#ifdef USE_IRIDESCENCE
void computeMultiscatteringIridescence( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float iridescence, const in vec3 iridescenceF0, const in float roughness, inout vec3 singleScatter, inout vec3 multiScatter ) {
#else
void computeMultiscattering( const in vec3 normal, const in vec3 viewDir, const in vec3 specularColor, const in float specularF90, const in float roughness, inout vec3 singleScatter, inout vec3 multiScatter ) {
#endif
	vec2 fab = DFGApprox( normal, viewDir, roughness );
	#ifdef USE_IRIDESCENCE
		vec3 Fr = mix( specularColor, iridescenceF0, iridescence );
	#else
		vec3 Fr = specularColor;
	#endif
	vec3 FssEss = Fr * fab.x + specularF90 * fab.y;
	float Ess = fab.x + fab.y;
	float Ems = 1.0 - Ess;
	vec3 Favg = Fr + ( 1.0 - Fr ) * 0.047619;	vec3 Fms = FssEss * Favg / ( 1.0 - Ems * Favg );
	singleScatter += FssEss;
	multiScatter += Fms * Ems;
}
#if NUM_RECT_AREA_LIGHTS > 0
	void RE_Direct_RectArea_Physical( const in RectAreaLight rectAreaLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
		vec3 normal = geometryNormal;
		vec3 viewDir = geometryViewDir;
		vec3 position = geometryPosition;
		vec3 lightPos = rectAreaLight.position;
		vec3 halfWidth = rectAreaLight.halfWidth;
		vec3 halfHeight = rectAreaLight.halfHeight;
		vec3 lightColor = rectAreaLight.color;
		float roughness = material.roughness;
		vec3 rectCoords[ 4 ];
		rectCoords[ 0 ] = lightPos + halfWidth - halfHeight;		rectCoords[ 1 ] = lightPos - halfWidth - halfHeight;
		rectCoords[ 2 ] = lightPos - halfWidth + halfHeight;
		rectCoords[ 3 ] = lightPos + halfWidth + halfHeight;
		vec2 uv = LTC_Uv( normal, viewDir, roughness );
		vec4 t1 = texture2D( ltc_1, uv );
		vec4 t2 = texture2D( ltc_2, uv );
		mat3 mInv = mat3(
			vec3( t1.x, 0, t1.y ),
			vec3(    0, 1,    0 ),
			vec3( t1.z, 0, t1.w )
		);
		vec3 fresnel = ( material.specularColor * t2.x + ( vec3( 1.0 ) - material.specularColor ) * t2.y );
		reflectedLight.directSpecular += lightColor * fresnel * LTC_Evaluate( normal, viewDir, position, mInv, rectCoords );
		reflectedLight.directDiffuse += lightColor * material.diffuseColor * LTC_Evaluate( normal, viewDir, position, mat3( 1.0 ), rectCoords );
	}
#endif
void RE_Direct_Physical( const in IncidentLight directLight, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
	float dotNL = saturate( dot( geometryNormal, directLight.direction ) );
	vec3 irradiance = dotNL * directLight.color;
	#ifdef USE_CLEARCOAT
		float dotNLcc = saturate( dot( geometryClearcoatNormal, directLight.direction ) );
		vec3 ccIrradiance = dotNLcc * directLight.color;
		clearcoatSpecularDirect += ccIrradiance * BRDF_GGX_Clearcoat( directLight.direction, geometryViewDir, geometryClearcoatNormal, material );
	#endif
	#ifdef USE_SHEEN
		sheenSpecularDirect += irradiance * BRDF_Sheen( directLight.direction, geometryViewDir, geometryNormal, material.sheenColor, material.sheenRoughness );
	#endif
	reflectedLight.directSpecular += irradiance * BRDF_GGX( directLight.direction, geometryViewDir, geometryNormal, material );
	reflectedLight.directDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectDiffuse_Physical( const in vec3 irradiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight ) {
	reflectedLight.indirectDiffuse += irradiance * BRDF_Lambert( material.diffuseColor );
}
void RE_IndirectSpecular_Physical( const in vec3 radiance, const in vec3 irradiance, const in vec3 clearcoatRadiance, const in vec3 geometryPosition, const in vec3 geometryNormal, const in vec3 geometryViewDir, const in vec3 geometryClearcoatNormal, const in PhysicalMaterial material, inout ReflectedLight reflectedLight) {
	#ifdef USE_CLEARCOAT
		clearcoatSpecularIndirect += clearcoatRadiance * EnvironmentBRDF( geometryClearcoatNormal, geometryViewDir, material.clearcoatF0, material.clearcoatF90, material.clearcoatRoughness );
	#endif
	#ifdef USE_SHEEN
		sheenSpecularIndirect += irradiance * material.sheenColor * IBLSheenBRDF( geometryNormal, geometryViewDir, material.sheenRoughness );
	#endif
	vec3 singleScattering = vec3( 0.0 );
	vec3 multiScattering = vec3( 0.0 );
	vec3 cosineWeightedIrradiance = irradiance * RECIPROCAL_PI;
	#ifdef USE_IRIDESCENCE
		computeMultiscatteringIridescence( geometryNormal, geometryViewDir, material.specularColor, material.specularF90, material.iridescence, material.iridescenceFresnel, material.roughness, singleScattering, multiScattering );
	#else
		computeMultiscattering( geometryNormal, geometryViewDir, material.specularColor, material.specularF90, material.roughness, singleScattering, multiScattering );
	#endif
	vec3 totalScattering = singleScattering + multiScattering;
	vec3 diffuse = material.diffuseColor * ( 1.0 - max( max( totalScattering.r, totalScattering.g ), totalScattering.b ) );
	reflectedLight.indirectSpecular += radiance * singleScattering;
	reflectedLight.indirectSpecular += multiScattering * cosineWeightedIrradiance;
	reflectedLight.indirectDiffuse += diffuse * cosineWeightedIrradiance;
}
#define RE_Direct				RE_Direct_Physical
#define RE_Direct_RectArea		RE_Direct_RectArea_Physical
#define RE_IndirectDiffuse		RE_IndirectDiffuse_Physical
#define RE_IndirectSpecular		RE_IndirectSpecular_Physical
float computeSpecularOcclusion( const in float dotNV, const in float ambientOcclusion, const in float roughness ) {
	return saturate( pow( dotNV + ambientOcclusion, exp2( - 16.0 * roughness - 1.0 ) ) - 1.0 + ambientOcclusion );
}`,OT=`
vec3 geometryPosition = - vViewPosition;
vec3 geometryNormal = normal;
vec3 geometryViewDir = ( isOrthographic ) ? vec3( 0, 0, 1 ) : normalize( vViewPosition );
vec3 geometryClearcoatNormal = vec3( 0.0 );
#ifdef USE_CLEARCOAT
	geometryClearcoatNormal = clearcoatNormal;
#endif
#ifdef USE_IRIDESCENCE
	float dotNVi = saturate( dot( normal, geometryViewDir ) );
	if ( material.iridescenceThickness == 0.0 ) {
		material.iridescence = 0.0;
	} else {
		material.iridescence = saturate( material.iridescence );
	}
	if ( material.iridescence > 0.0 ) {
		material.iridescenceFresnel = evalIridescence( 1.0, material.iridescenceIOR, dotNVi, material.iridescenceThickness, material.specularColor );
		material.iridescenceF0 = Schlick_to_F0( material.iridescenceFresnel, 1.0, dotNVi );
	}
#endif
IncidentLight directLight;
#if ( NUM_POINT_LIGHTS > 0 ) && defined( RE_Direct )
	PointLight pointLight;
	#if defined( USE_SHADOWMAP ) && NUM_POINT_LIGHT_SHADOWS > 0
	PointLightShadow pointLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_POINT_LIGHTS; i ++ ) {
		pointLight = pointLights[ i ];
		getPointLightInfo( pointLight, geometryPosition, directLight );
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_POINT_LIGHT_SHADOWS )
		pointLightShadow = pointLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getPointShadow( pointShadowMap[ i ], pointLightShadow.shadowMapSize, pointLightShadow.shadowIntensity, pointLightShadow.shadowBias, pointLightShadow.shadowRadius, vPointShadowCoord[ i ], pointLightShadow.shadowCameraNear, pointLightShadow.shadowCameraFar ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_SPOT_LIGHTS > 0 ) && defined( RE_Direct )
	SpotLight spotLight;
	vec4 spotColor;
	vec3 spotLightCoord;
	bool inSpotLightMap;
	#if defined( USE_SHADOWMAP ) && NUM_SPOT_LIGHT_SHADOWS > 0
	SpotLightShadow spotLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHTS; i ++ ) {
		spotLight = spotLights[ i ];
		getSpotLightInfo( spotLight, geometryPosition, directLight );
		#if ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS )
		#define SPOT_LIGHT_MAP_INDEX UNROLLED_LOOP_INDEX
		#elif ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
		#define SPOT_LIGHT_MAP_INDEX NUM_SPOT_LIGHT_MAPS
		#else
		#define SPOT_LIGHT_MAP_INDEX ( UNROLLED_LOOP_INDEX - NUM_SPOT_LIGHT_SHADOWS + NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS )
		#endif
		#if ( SPOT_LIGHT_MAP_INDEX < NUM_SPOT_LIGHT_MAPS )
			spotLightCoord = vSpotLightCoord[ i ].xyz / vSpotLightCoord[ i ].w;
			inSpotLightMap = all( lessThan( abs( spotLightCoord * 2. - 1. ), vec3( 1.0 ) ) );
			spotColor = texture2D( spotLightMap[ SPOT_LIGHT_MAP_INDEX ], spotLightCoord.xy );
			directLight.color = inSpotLightMap ? directLight.color * spotColor.rgb : directLight.color;
		#endif
		#undef SPOT_LIGHT_MAP_INDEX
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
		spotLightShadow = spotLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getShadow( spotShadowMap[ i ], spotLightShadow.shadowMapSize, spotLightShadow.shadowIntensity, spotLightShadow.shadowBias, spotLightShadow.shadowRadius, vSpotLightCoord[ i ] ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_DIR_LIGHTS > 0 ) && defined( RE_Direct )
	DirectionalLight directionalLight;
	#if defined( USE_SHADOWMAP ) && NUM_DIR_LIGHT_SHADOWS > 0
	DirectionalLightShadow directionalLightShadow;
	#endif
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_DIR_LIGHTS; i ++ ) {
		directionalLight = directionalLights[ i ];
		getDirectionalLightInfo( directionalLight, directLight );
		#if defined( USE_SHADOWMAP ) && ( UNROLLED_LOOP_INDEX < NUM_DIR_LIGHT_SHADOWS )
		directionalLightShadow = directionalLightShadows[ i ];
		directLight.color *= ( directLight.visible && receiveShadow ) ? getShadow( directionalShadowMap[ i ], directionalLightShadow.shadowMapSize, directionalLightShadow.shadowIntensity, directionalLightShadow.shadowBias, directionalLightShadow.shadowRadius, vDirectionalShadowCoord[ i ] ) : 1.0;
		#endif
		RE_Direct( directLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if ( NUM_RECT_AREA_LIGHTS > 0 ) && defined( RE_Direct_RectArea )
	RectAreaLight rectAreaLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_RECT_AREA_LIGHTS; i ++ ) {
		rectAreaLight = rectAreaLights[ i ];
		RE_Direct_RectArea( rectAreaLight, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
	}
	#pragma unroll_loop_end
#endif
#if defined( RE_IndirectDiffuse )
	vec3 iblIrradiance = vec3( 0.0 );
	vec3 irradiance = getAmbientLightIrradiance( ambientLightColor );
	#if defined( USE_LIGHT_PROBES )
		irradiance += getLightProbeIrradiance( lightProbe, geometryNormal );
	#endif
	#if ( NUM_HEMI_LIGHTS > 0 )
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_HEMI_LIGHTS; i ++ ) {
			irradiance += getHemisphereLightIrradiance( hemisphereLights[ i ], geometryNormal );
		}
		#pragma unroll_loop_end
	#endif
#endif
#if defined( RE_IndirectSpecular )
	vec3 radiance = vec3( 0.0 );
	vec3 clearcoatRadiance = vec3( 0.0 );
#endif`,kT=`#if defined( RE_IndirectDiffuse )
	#ifdef USE_LIGHTMAP
		vec4 lightMapTexel = texture2D( lightMap, vLightMapUv );
		vec3 lightMapIrradiance = lightMapTexel.rgb * lightMapIntensity;
		irradiance += lightMapIrradiance;
	#endif
	#if defined( USE_ENVMAP ) && defined( STANDARD ) && defined( ENVMAP_TYPE_CUBE_UV )
		iblIrradiance += getIBLIrradiance( geometryNormal );
	#endif
#endif
#if defined( USE_ENVMAP ) && defined( RE_IndirectSpecular )
	#ifdef USE_ANISOTROPY
		radiance += getIBLAnisotropyRadiance( geometryViewDir, geometryNormal, material.roughness, material.anisotropyB, material.anisotropy );
	#else
		radiance += getIBLRadiance( geometryViewDir, geometryNormal, material.roughness );
	#endif
	#ifdef USE_CLEARCOAT
		clearcoatRadiance += getIBLRadiance( geometryViewDir, geometryClearcoatNormal, material.clearcoatRoughness );
	#endif
#endif`,BT=`#if defined( RE_IndirectDiffuse )
	RE_IndirectDiffuse( irradiance, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
#endif
#if defined( RE_IndirectSpecular )
	RE_IndirectSpecular( radiance, iblIrradiance, clearcoatRadiance, geometryPosition, geometryNormal, geometryViewDir, geometryClearcoatNormal, material, reflectedLight );
#endif`,zT=`#if defined( USE_LOGDEPTHBUF )
	gl_FragDepth = vIsPerspective == 0.0 ? gl_FragCoord.z : log2( vFragDepth ) * logDepthBufFC * 0.5;
#endif`,HT=`#if defined( USE_LOGDEPTHBUF )
	uniform float logDepthBufFC;
	varying float vFragDepth;
	varying float vIsPerspective;
#endif`,VT=`#ifdef USE_LOGDEPTHBUF
	varying float vFragDepth;
	varying float vIsPerspective;
#endif`,GT=`#ifdef USE_LOGDEPTHBUF
	vFragDepth = 1.0 + gl_Position.w;
	vIsPerspective = float( isPerspectiveMatrix( projectionMatrix ) );
#endif`,WT=`#ifdef USE_MAP
	vec4 sampledDiffuseColor = texture2D( map, vMapUv );
	#ifdef DECODE_VIDEO_TEXTURE
		sampledDiffuseColor = vec4( mix( pow( sampledDiffuseColor.rgb * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) ), sampledDiffuseColor.rgb * 0.0773993808, vec3( lessThanEqual( sampledDiffuseColor.rgb, vec3( 0.04045 ) ) ) ), sampledDiffuseColor.w );
	
	#endif
	diffuseColor *= sampledDiffuseColor;
#endif`,XT=`#ifdef USE_MAP
	uniform sampler2D map;
#endif`,jT=`#if defined( USE_MAP ) || defined( USE_ALPHAMAP )
	#if defined( USE_POINTS_UV )
		vec2 uv = vUv;
	#else
		vec2 uv = ( uvTransform * vec3( gl_PointCoord.x, 1.0 - gl_PointCoord.y, 1 ) ).xy;
	#endif
#endif
#ifdef USE_MAP
	diffuseColor *= texture2D( map, uv );
#endif
#ifdef USE_ALPHAMAP
	diffuseColor.a *= texture2D( alphaMap, uv ).g;
#endif`,YT=`#if defined( USE_POINTS_UV )
	varying vec2 vUv;
#else
	#if defined( USE_MAP ) || defined( USE_ALPHAMAP )
		uniform mat3 uvTransform;
	#endif
#endif
#ifdef USE_MAP
	uniform sampler2D map;
#endif
#ifdef USE_ALPHAMAP
	uniform sampler2D alphaMap;
#endif`,KT=`float metalnessFactor = metalness;
#ifdef USE_METALNESSMAP
	vec4 texelMetalness = texture2D( metalnessMap, vMetalnessMapUv );
	metalnessFactor *= texelMetalness.b;
#endif`,qT=`#ifdef USE_METALNESSMAP
	uniform sampler2D metalnessMap;
#endif`,$T=`#ifdef USE_INSTANCING_MORPH
	float morphTargetInfluences[ MORPHTARGETS_COUNT ];
	float morphTargetBaseInfluence = texelFetch( morphTexture, ivec2( 0, gl_InstanceID ), 0 ).r;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		morphTargetInfluences[i] =  texelFetch( morphTexture, ivec2( i + 1, gl_InstanceID ), 0 ).r;
	}
#endif`,ZT=`#if defined( USE_MORPHCOLORS )
	vColor *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		#if defined( USE_COLOR_ALPHA )
			if ( morphTargetInfluences[ i ] != 0.0 ) vColor += getMorph( gl_VertexID, i, 2 ) * morphTargetInfluences[ i ];
		#elif defined( USE_COLOR )
			if ( morphTargetInfluences[ i ] != 0.0 ) vColor += getMorph( gl_VertexID, i, 2 ).rgb * morphTargetInfluences[ i ];
		#endif
	}
#endif`,QT=`#ifdef USE_MORPHNORMALS
	objectNormal *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		if ( morphTargetInfluences[ i ] != 0.0 ) objectNormal += getMorph( gl_VertexID, i, 1 ).xyz * morphTargetInfluences[ i ];
	}
#endif`,JT=`#ifdef USE_MORPHTARGETS
	#ifndef USE_INSTANCING_MORPH
		uniform float morphTargetBaseInfluence;
		uniform float morphTargetInfluences[ MORPHTARGETS_COUNT ];
	#endif
	uniform sampler2DArray morphTargetsTexture;
	uniform ivec2 morphTargetsTextureSize;
	vec4 getMorph( const in int vertexIndex, const in int morphTargetIndex, const in int offset ) {
		int texelIndex = vertexIndex * MORPHTARGETS_TEXTURE_STRIDE + offset;
		int y = texelIndex / morphTargetsTextureSize.x;
		int x = texelIndex - y * morphTargetsTextureSize.x;
		ivec3 morphUV = ivec3( x, y, morphTargetIndex );
		return texelFetch( morphTargetsTexture, morphUV, 0 );
	}
#endif`,ew=`#ifdef USE_MORPHTARGETS
	transformed *= morphTargetBaseInfluence;
	for ( int i = 0; i < MORPHTARGETS_COUNT; i ++ ) {
		if ( morphTargetInfluences[ i ] != 0.0 ) transformed += getMorph( gl_VertexID, i, 0 ).xyz * morphTargetInfluences[ i ];
	}
#endif`,tw=`float faceDirection = gl_FrontFacing ? 1.0 : - 1.0;
#ifdef FLAT_SHADED
	vec3 fdx = dFdx( vViewPosition );
	vec3 fdy = dFdy( vViewPosition );
	vec3 normal = normalize( cross( fdx, fdy ) );
#else
	vec3 normal = normalize( vNormal );
	#ifdef DOUBLE_SIDED
		normal *= faceDirection;
	#endif
#endif
#if defined( USE_NORMALMAP_TANGENTSPACE ) || defined( USE_CLEARCOAT_NORMALMAP ) || defined( USE_ANISOTROPY )
	#ifdef USE_TANGENT
		mat3 tbn = mat3( normalize( vTangent ), normalize( vBitangent ), normal );
	#else
		mat3 tbn = getTangentFrame( - vViewPosition, normal,
		#if defined( USE_NORMALMAP )
			vNormalMapUv
		#elif defined( USE_CLEARCOAT_NORMALMAP )
			vClearcoatNormalMapUv
		#else
			vUv
		#endif
		);
	#endif
	#if defined( DOUBLE_SIDED ) && ! defined( FLAT_SHADED )
		tbn[0] *= faceDirection;
		tbn[1] *= faceDirection;
	#endif
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	#ifdef USE_TANGENT
		mat3 tbn2 = mat3( normalize( vTangent ), normalize( vBitangent ), normal );
	#else
		mat3 tbn2 = getTangentFrame( - vViewPosition, normal, vClearcoatNormalMapUv );
	#endif
	#if defined( DOUBLE_SIDED ) && ! defined( FLAT_SHADED )
		tbn2[0] *= faceDirection;
		tbn2[1] *= faceDirection;
	#endif
#endif
vec3 nonPerturbedNormal = normal;`,nw=`#ifdef USE_NORMALMAP_OBJECTSPACE
	normal = texture2D( normalMap, vNormalMapUv ).xyz * 2.0 - 1.0;
	#ifdef FLIP_SIDED
		normal = - normal;
	#endif
	#ifdef DOUBLE_SIDED
		normal = normal * faceDirection;
	#endif
	normal = normalize( normalMatrix * normal );
#elif defined( USE_NORMALMAP_TANGENTSPACE )
	vec3 mapN = texture2D( normalMap, vNormalMapUv ).xyz * 2.0 - 1.0;
	mapN.xy *= normalScale;
	normal = normalize( tbn * mapN );
#elif defined( USE_BUMPMAP )
	normal = perturbNormalArb( - vViewPosition, normal, dHdxy_fwd(), faceDirection );
#endif`,iw=`#ifndef FLAT_SHADED
	varying vec3 vNormal;
	#ifdef USE_TANGENT
		varying vec3 vTangent;
		varying vec3 vBitangent;
	#endif
#endif`,rw=`#ifndef FLAT_SHADED
	varying vec3 vNormal;
	#ifdef USE_TANGENT
		varying vec3 vTangent;
		varying vec3 vBitangent;
	#endif
#endif`,sw=`#ifndef FLAT_SHADED
	vNormal = normalize( transformedNormal );
	#ifdef USE_TANGENT
		vTangent = normalize( transformedTangent );
		vBitangent = normalize( cross( vNormal, vTangent ) * tangent.w );
	#endif
#endif`,ow=`#ifdef USE_NORMALMAP
	uniform sampler2D normalMap;
	uniform vec2 normalScale;
#endif
#ifdef USE_NORMALMAP_OBJECTSPACE
	uniform mat3 normalMatrix;
#endif
#if ! defined ( USE_TANGENT ) && ( defined ( USE_NORMALMAP_TANGENTSPACE ) || defined ( USE_CLEARCOAT_NORMALMAP ) || defined( USE_ANISOTROPY ) )
	mat3 getTangentFrame( vec3 eye_pos, vec3 surf_norm, vec2 uv ) {
		vec3 q0 = dFdx( eye_pos.xyz );
		vec3 q1 = dFdy( eye_pos.xyz );
		vec2 st0 = dFdx( uv.st );
		vec2 st1 = dFdy( uv.st );
		vec3 N = surf_norm;
		vec3 q1perp = cross( q1, N );
		vec3 q0perp = cross( N, q0 );
		vec3 T = q1perp * st0.x + q0perp * st1.x;
		vec3 B = q1perp * st0.y + q0perp * st1.y;
		float det = max( dot( T, T ), dot( B, B ) );
		float scale = ( det == 0.0 ) ? 0.0 : inversesqrt( det );
		return mat3( T * scale, B * scale, N );
	}
#endif`,aw=`#ifdef USE_CLEARCOAT
	vec3 clearcoatNormal = nonPerturbedNormal;
#endif`,lw=`#ifdef USE_CLEARCOAT_NORMALMAP
	vec3 clearcoatMapN = texture2D( clearcoatNormalMap, vClearcoatNormalMapUv ).xyz * 2.0 - 1.0;
	clearcoatMapN.xy *= clearcoatNormalScale;
	clearcoatNormal = normalize( tbn2 * clearcoatMapN );
#endif`,cw=`#ifdef USE_CLEARCOATMAP
	uniform sampler2D clearcoatMap;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	uniform sampler2D clearcoatNormalMap;
	uniform vec2 clearcoatNormalScale;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	uniform sampler2D clearcoatRoughnessMap;
#endif`,uw=`#ifdef USE_IRIDESCENCEMAP
	uniform sampler2D iridescenceMap;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	uniform sampler2D iridescenceThicknessMap;
#endif`,dw=`#ifdef OPAQUE
diffuseColor.a = 1.0;
#endif
#ifdef USE_TRANSMISSION
diffuseColor.a *= material.transmissionAlpha;
#endif
gl_FragColor = vec4( outgoingLight, diffuseColor.a );`,fw=`vec3 packNormalToRGB( const in vec3 normal ) {
	return normalize( normal ) * 0.5 + 0.5;
}
vec3 unpackRGBToNormal( const in vec3 rgb ) {
	return 2.0 * rgb.xyz - 1.0;
}
const float PackUpscale = 256. / 255.;const float UnpackDownscale = 255. / 256.;const float ShiftRight8 = 1. / 256.;
const float Inv255 = 1. / 255.;
const vec4 PackFactors = vec4( 1.0, 256.0, 256.0 * 256.0, 256.0 * 256.0 * 256.0 );
const vec2 UnpackFactors2 = vec2( UnpackDownscale, 1.0 / PackFactors.g );
const vec3 UnpackFactors3 = vec3( UnpackDownscale / PackFactors.rg, 1.0 / PackFactors.b );
const vec4 UnpackFactors4 = vec4( UnpackDownscale / PackFactors.rgb, 1.0 / PackFactors.a );
vec4 packDepthToRGBA( const in float v ) {
	if( v <= 0.0 )
		return vec4( 0., 0., 0., 0. );
	if( v >= 1.0 )
		return vec4( 1., 1., 1., 1. );
	float vuf;
	float af = modf( v * PackFactors.a, vuf );
	float bf = modf( vuf * ShiftRight8, vuf );
	float gf = modf( vuf * ShiftRight8, vuf );
	return vec4( vuf * Inv255, gf * PackUpscale, bf * PackUpscale, af );
}
vec3 packDepthToRGB( const in float v ) {
	if( v <= 0.0 )
		return vec3( 0., 0., 0. );
	if( v >= 1.0 )
		return vec3( 1., 1., 1. );
	float vuf;
	float bf = modf( v * PackFactors.b, vuf );
	float gf = modf( vuf * ShiftRight8, vuf );
	return vec3( vuf * Inv255, gf * PackUpscale, bf );
}
vec2 packDepthToRG( const in float v ) {
	if( v <= 0.0 )
		return vec2( 0., 0. );
	if( v >= 1.0 )
		return vec2( 1., 1. );
	float vuf;
	float gf = modf( v * 256., vuf );
	return vec2( vuf * Inv255, gf );
}
float unpackRGBAToDepth( const in vec4 v ) {
	return dot( v, UnpackFactors4 );
}
float unpackRGBToDepth( const in vec3 v ) {
	return dot( v, UnpackFactors3 );
}
float unpackRGToDepth( const in vec2 v ) {
	return v.r * UnpackFactors2.r + v.g * UnpackFactors2.g;
}
vec4 pack2HalfToRGBA( const in vec2 v ) {
	vec4 r = vec4( v.x, fract( v.x * 255.0 ), v.y, fract( v.y * 255.0 ) );
	return vec4( r.x - r.y / 255.0, r.y, r.z - r.w / 255.0, r.w );
}
vec2 unpackRGBATo2Half( const in vec4 v ) {
	return vec2( v.x + ( v.y / 255.0 ), v.z + ( v.w / 255.0 ) );
}
float viewZToOrthographicDepth( const in float viewZ, const in float near, const in float far ) {
	return ( viewZ + near ) / ( near - far );
}
float orthographicDepthToViewZ( const in float depth, const in float near, const in float far ) {
	return depth * ( near - far ) - near;
}
float viewZToPerspectiveDepth( const in float viewZ, const in float near, const in float far ) {
	return ( ( near + viewZ ) * far ) / ( ( far - near ) * viewZ );
}
float perspectiveDepthToViewZ( const in float depth, const in float near, const in float far ) {
	return ( near * far ) / ( ( far - near ) * depth - far );
}`,hw=`#ifdef PREMULTIPLIED_ALPHA
	gl_FragColor.rgb *= gl_FragColor.a;
#endif`,pw=`vec4 mvPosition = vec4( transformed, 1.0 );
#ifdef USE_BATCHING
	mvPosition = batchingMatrix * mvPosition;
#endif
#ifdef USE_INSTANCING
	mvPosition = instanceMatrix * mvPosition;
#endif
mvPosition = modelViewMatrix * mvPosition;
gl_Position = projectionMatrix * mvPosition;`,mw=`#ifdef DITHERING
	gl_FragColor.rgb = dithering( gl_FragColor.rgb );
#endif`,gw=`#ifdef DITHERING
	vec3 dithering( vec3 color ) {
		float grid_position = rand( gl_FragCoord.xy );
		vec3 dither_shift_RGB = vec3( 0.25 / 255.0, -0.25 / 255.0, 0.25 / 255.0 );
		dither_shift_RGB = mix( 2.0 * dither_shift_RGB, -2.0 * dither_shift_RGB, grid_position );
		return color + dither_shift_RGB;
	}
#endif`,_w=`float roughnessFactor = roughness;
#ifdef USE_ROUGHNESSMAP
	vec4 texelRoughness = texture2D( roughnessMap, vRoughnessMapUv );
	roughnessFactor *= texelRoughness.g;
#endif`,vw=`#ifdef USE_ROUGHNESSMAP
	uniform sampler2D roughnessMap;
#endif`,yw=`#if NUM_SPOT_LIGHT_COORDS > 0
	varying vec4 vSpotLightCoord[ NUM_SPOT_LIGHT_COORDS ];
#endif
#if NUM_SPOT_LIGHT_MAPS > 0
	uniform sampler2D spotLightMap[ NUM_SPOT_LIGHT_MAPS ];
#endif
#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
		uniform sampler2D directionalShadowMap[ NUM_DIR_LIGHT_SHADOWS ];
		varying vec4 vDirectionalShadowCoord[ NUM_DIR_LIGHT_SHADOWS ];
		struct DirectionalLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform DirectionalLightShadow directionalLightShadows[ NUM_DIR_LIGHT_SHADOWS ];
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
		uniform sampler2D spotShadowMap[ NUM_SPOT_LIGHT_SHADOWS ];
		struct SpotLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform SpotLightShadow spotLightShadows[ NUM_SPOT_LIGHT_SHADOWS ];
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		uniform sampler2D pointShadowMap[ NUM_POINT_LIGHT_SHADOWS ];
		varying vec4 vPointShadowCoord[ NUM_POINT_LIGHT_SHADOWS ];
		struct PointLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
			float shadowCameraNear;
			float shadowCameraFar;
		};
		uniform PointLightShadow pointLightShadows[ NUM_POINT_LIGHT_SHADOWS ];
	#endif
	float texture2DCompare( sampler2D depths, vec2 uv, float compare ) {
		return step( compare, unpackRGBAToDepth( texture2D( depths, uv ) ) );
	}
	vec2 texture2DDistribution( sampler2D shadow, vec2 uv ) {
		return unpackRGBATo2Half( texture2D( shadow, uv ) );
	}
	float VSMShadow (sampler2D shadow, vec2 uv, float compare ){
		float occlusion = 1.0;
		vec2 distribution = texture2DDistribution( shadow, uv );
		float hard_shadow = step( compare , distribution.x );
		if (hard_shadow != 1.0 ) {
			float distance = compare - distribution.x ;
			float variance = max( 0.00000, distribution.y * distribution.y );
			float softness_probability = variance / (variance + distance * distance );			softness_probability = clamp( ( softness_probability - 0.3 ) / ( 0.95 - 0.3 ), 0.0, 1.0 );			occlusion = clamp( max( hard_shadow, softness_probability ), 0.0, 1.0 );
		}
		return occlusion;
	}
	float getShadow( sampler2D shadowMap, vec2 shadowMapSize, float shadowIntensity, float shadowBias, float shadowRadius, vec4 shadowCoord ) {
		float shadow = 1.0;
		shadowCoord.xyz /= shadowCoord.w;
		shadowCoord.z += shadowBias;
		bool inFrustum = shadowCoord.x >= 0.0 && shadowCoord.x <= 1.0 && shadowCoord.y >= 0.0 && shadowCoord.y <= 1.0;
		bool frustumTest = inFrustum && shadowCoord.z <= 1.0;
		if ( frustumTest ) {
		#if defined( SHADOWMAP_TYPE_PCF )
			vec2 texelSize = vec2( 1.0 ) / shadowMapSize;
			float dx0 = - texelSize.x * shadowRadius;
			float dy0 = - texelSize.y * shadowRadius;
			float dx1 = + texelSize.x * shadowRadius;
			float dy1 = + texelSize.y * shadowRadius;
			float dx2 = dx0 / 2.0;
			float dy2 = dy0 / 2.0;
			float dx3 = dx1 / 2.0;
			float dy3 = dy1 / 2.0;
			shadow = (
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx0, dy0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx1, dy0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx2, dy2 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy2 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx3, dy2 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx0, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx2, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy, shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx3, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx1, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx2, dy3 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy3 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx3, dy3 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx0, dy1 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( 0.0, dy1 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, shadowCoord.xy + vec2( dx1, dy1 ), shadowCoord.z )
			) * ( 1.0 / 17.0 );
		#elif defined( SHADOWMAP_TYPE_PCF_SOFT )
			vec2 texelSize = vec2( 1.0 ) / shadowMapSize;
			float dx = texelSize.x;
			float dy = texelSize.y;
			vec2 uv = shadowCoord.xy;
			vec2 f = fract( uv * shadowMapSize + 0.5 );
			uv -= f * texelSize;
			shadow = (
				texture2DCompare( shadowMap, uv, shadowCoord.z ) +
				texture2DCompare( shadowMap, uv + vec2( dx, 0.0 ), shadowCoord.z ) +
				texture2DCompare( shadowMap, uv + vec2( 0.0, dy ), shadowCoord.z ) +
				texture2DCompare( shadowMap, uv + texelSize, shadowCoord.z ) +
				mix( texture2DCompare( shadowMap, uv + vec2( -dx, 0.0 ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, 0.0 ), shadowCoord.z ),
					 f.x ) +
				mix( texture2DCompare( shadowMap, uv + vec2( -dx, dy ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, dy ), shadowCoord.z ),
					 f.x ) +
				mix( texture2DCompare( shadowMap, uv + vec2( 0.0, -dy ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( 0.0, 2.0 * dy ), shadowCoord.z ),
					 f.y ) +
				mix( texture2DCompare( shadowMap, uv + vec2( dx, -dy ), shadowCoord.z ),
					 texture2DCompare( shadowMap, uv + vec2( dx, 2.0 * dy ), shadowCoord.z ),
					 f.y ) +
				mix( mix( texture2DCompare( shadowMap, uv + vec2( -dx, -dy ), shadowCoord.z ),
						  texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, -dy ), shadowCoord.z ),
						  f.x ),
					 mix( texture2DCompare( shadowMap, uv + vec2( -dx, 2.0 * dy ), shadowCoord.z ),
						  texture2DCompare( shadowMap, uv + vec2( 2.0 * dx, 2.0 * dy ), shadowCoord.z ),
						  f.x ),
					 f.y )
			) * ( 1.0 / 9.0 );
		#elif defined( SHADOWMAP_TYPE_VSM )
			shadow = VSMShadow( shadowMap, shadowCoord.xy, shadowCoord.z );
		#else
			shadow = texture2DCompare( shadowMap, shadowCoord.xy, shadowCoord.z );
		#endif
		}
		return mix( 1.0, shadow, shadowIntensity );
	}
	vec2 cubeToUV( vec3 v, float texelSizeY ) {
		vec3 absV = abs( v );
		float scaleToCube = 1.0 / max( absV.x, max( absV.y, absV.z ) );
		absV *= scaleToCube;
		v *= scaleToCube * ( 1.0 - 2.0 * texelSizeY );
		vec2 planar = v.xy;
		float almostATexel = 1.5 * texelSizeY;
		float almostOne = 1.0 - almostATexel;
		if ( absV.z >= almostOne ) {
			if ( v.z > 0.0 )
				planar.x = 4.0 - v.x;
		} else if ( absV.x >= almostOne ) {
			float signX = sign( v.x );
			planar.x = v.z * signX + 2.0 * signX;
		} else if ( absV.y >= almostOne ) {
			float signY = sign( v.y );
			planar.x = v.x + 2.0 * signY + 2.0;
			planar.y = v.z * signY - 2.0;
		}
		return vec2( 0.125, 0.25 ) * planar + vec2( 0.375, 0.75 );
	}
	float getPointShadow( sampler2D shadowMap, vec2 shadowMapSize, float shadowIntensity, float shadowBias, float shadowRadius, vec4 shadowCoord, float shadowCameraNear, float shadowCameraFar ) {
		float shadow = 1.0;
		vec3 lightToPosition = shadowCoord.xyz;
		
		float lightToPositionLength = length( lightToPosition );
		if ( lightToPositionLength - shadowCameraFar <= 0.0 && lightToPositionLength - shadowCameraNear >= 0.0 ) {
			float dp = ( lightToPositionLength - shadowCameraNear ) / ( shadowCameraFar - shadowCameraNear );			dp += shadowBias;
			vec3 bd3D = normalize( lightToPosition );
			vec2 texelSize = vec2( 1.0 ) / ( shadowMapSize * vec2( 4.0, 2.0 ) );
			#if defined( SHADOWMAP_TYPE_PCF ) || defined( SHADOWMAP_TYPE_PCF_SOFT ) || defined( SHADOWMAP_TYPE_VSM )
				vec2 offset = vec2( - 1, 1 ) * shadowRadius * texelSize.y;
				shadow = (
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xyy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yyy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xyx, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yyx, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xxy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yxy, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.xxx, texelSize.y ), dp ) +
					texture2DCompare( shadowMap, cubeToUV( bd3D + offset.yxx, texelSize.y ), dp )
				) * ( 1.0 / 9.0 );
			#else
				shadow = texture2DCompare( shadowMap, cubeToUV( bd3D, texelSize.y ), dp );
			#endif
		}
		return mix( 1.0, shadow, shadowIntensity );
	}
#endif`,xw=`#if NUM_SPOT_LIGHT_COORDS > 0
	uniform mat4 spotLightMatrix[ NUM_SPOT_LIGHT_COORDS ];
	varying vec4 vSpotLightCoord[ NUM_SPOT_LIGHT_COORDS ];
#endif
#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
		uniform mat4 directionalShadowMatrix[ NUM_DIR_LIGHT_SHADOWS ];
		varying vec4 vDirectionalShadowCoord[ NUM_DIR_LIGHT_SHADOWS ];
		struct DirectionalLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform DirectionalLightShadow directionalLightShadows[ NUM_DIR_LIGHT_SHADOWS ];
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
		struct SpotLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
		};
		uniform SpotLightShadow spotLightShadows[ NUM_SPOT_LIGHT_SHADOWS ];
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		uniform mat4 pointShadowMatrix[ NUM_POINT_LIGHT_SHADOWS ];
		varying vec4 vPointShadowCoord[ NUM_POINT_LIGHT_SHADOWS ];
		struct PointLightShadow {
			float shadowIntensity;
			float shadowBias;
			float shadowNormalBias;
			float shadowRadius;
			vec2 shadowMapSize;
			float shadowCameraNear;
			float shadowCameraFar;
		};
		uniform PointLightShadow pointLightShadows[ NUM_POINT_LIGHT_SHADOWS ];
	#endif
#endif`,Sw=`#if ( defined( USE_SHADOWMAP ) && ( NUM_DIR_LIGHT_SHADOWS > 0 || NUM_POINT_LIGHT_SHADOWS > 0 ) ) || ( NUM_SPOT_LIGHT_COORDS > 0 )
	vec3 shadowWorldNormal = inverseTransformDirection( transformedNormal, viewMatrix );
	vec4 shadowWorldPosition;
#endif
#if defined( USE_SHADOWMAP )
	#if NUM_DIR_LIGHT_SHADOWS > 0
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_DIR_LIGHT_SHADOWS; i ++ ) {
			shadowWorldPosition = worldPosition + vec4( shadowWorldNormal * directionalLightShadows[ i ].shadowNormalBias, 0 );
			vDirectionalShadowCoord[ i ] = directionalShadowMatrix[ i ] * shadowWorldPosition;
		}
		#pragma unroll_loop_end
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
		#pragma unroll_loop_start
		for ( int i = 0; i < NUM_POINT_LIGHT_SHADOWS; i ++ ) {
			shadowWorldPosition = worldPosition + vec4( shadowWorldNormal * pointLightShadows[ i ].shadowNormalBias, 0 );
			vPointShadowCoord[ i ] = pointShadowMatrix[ i ] * shadowWorldPosition;
		}
		#pragma unroll_loop_end
	#endif
#endif
#if NUM_SPOT_LIGHT_COORDS > 0
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHT_COORDS; i ++ ) {
		shadowWorldPosition = worldPosition;
		#if ( defined( USE_SHADOWMAP ) && UNROLLED_LOOP_INDEX < NUM_SPOT_LIGHT_SHADOWS )
			shadowWorldPosition.xyz += shadowWorldNormal * spotLightShadows[ i ].shadowNormalBias;
		#endif
		vSpotLightCoord[ i ] = spotLightMatrix[ i ] * shadowWorldPosition;
	}
	#pragma unroll_loop_end
#endif`,Mw=`float getShadowMask() {
	float shadow = 1.0;
	#ifdef USE_SHADOWMAP
	#if NUM_DIR_LIGHT_SHADOWS > 0
	DirectionalLightShadow directionalLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_DIR_LIGHT_SHADOWS; i ++ ) {
		directionalLight = directionalLightShadows[ i ];
		shadow *= receiveShadow ? getShadow( directionalShadowMap[ i ], directionalLight.shadowMapSize, directionalLight.shadowIntensity, directionalLight.shadowBias, directionalLight.shadowRadius, vDirectionalShadowCoord[ i ] ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#if NUM_SPOT_LIGHT_SHADOWS > 0
	SpotLightShadow spotLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_SPOT_LIGHT_SHADOWS; i ++ ) {
		spotLight = spotLightShadows[ i ];
		shadow *= receiveShadow ? getShadow( spotShadowMap[ i ], spotLight.shadowMapSize, spotLight.shadowIntensity, spotLight.shadowBias, spotLight.shadowRadius, vSpotLightCoord[ i ] ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#if NUM_POINT_LIGHT_SHADOWS > 0
	PointLightShadow pointLight;
	#pragma unroll_loop_start
	for ( int i = 0; i < NUM_POINT_LIGHT_SHADOWS; i ++ ) {
		pointLight = pointLightShadows[ i ];
		shadow *= receiveShadow ? getPointShadow( pointShadowMap[ i ], pointLight.shadowMapSize, pointLight.shadowIntensity, pointLight.shadowBias, pointLight.shadowRadius, vPointShadowCoord[ i ], pointLight.shadowCameraNear, pointLight.shadowCameraFar ) : 1.0;
	}
	#pragma unroll_loop_end
	#endif
	#endif
	return shadow;
}`,Ew=`#ifdef USE_SKINNING
	mat4 boneMatX = getBoneMatrix( skinIndex.x );
	mat4 boneMatY = getBoneMatrix( skinIndex.y );
	mat4 boneMatZ = getBoneMatrix( skinIndex.z );
	mat4 boneMatW = getBoneMatrix( skinIndex.w );
#endif`,Tw=`#ifdef USE_SKINNING
	uniform mat4 bindMatrix;
	uniform mat4 bindMatrixInverse;
	uniform highp sampler2D boneTexture;
	mat4 getBoneMatrix( const in float i ) {
		int size = textureSize( boneTexture, 0 ).x;
		int j = int( i ) * 4;
		int x = j % size;
		int y = j / size;
		vec4 v1 = texelFetch( boneTexture, ivec2( x, y ), 0 );
		vec4 v2 = texelFetch( boneTexture, ivec2( x + 1, y ), 0 );
		vec4 v3 = texelFetch( boneTexture, ivec2( x + 2, y ), 0 );
		vec4 v4 = texelFetch( boneTexture, ivec2( x + 3, y ), 0 );
		return mat4( v1, v2, v3, v4 );
	}
#endif`,ww=`#ifdef USE_SKINNING
	vec4 skinVertex = bindMatrix * vec4( transformed, 1.0 );
	vec4 skinned = vec4( 0.0 );
	skinned += boneMatX * skinVertex * skinWeight.x;
	skinned += boneMatY * skinVertex * skinWeight.y;
	skinned += boneMatZ * skinVertex * skinWeight.z;
	skinned += boneMatW * skinVertex * skinWeight.w;
	transformed = ( bindMatrixInverse * skinned ).xyz;
#endif`,Aw=`#ifdef USE_SKINNING
	mat4 skinMatrix = mat4( 0.0 );
	skinMatrix += skinWeight.x * boneMatX;
	skinMatrix += skinWeight.y * boneMatY;
	skinMatrix += skinWeight.z * boneMatZ;
	skinMatrix += skinWeight.w * boneMatW;
	skinMatrix = bindMatrixInverse * skinMatrix * bindMatrix;
	objectNormal = vec4( skinMatrix * vec4( objectNormal, 0.0 ) ).xyz;
	#ifdef USE_TANGENT
		objectTangent = vec4( skinMatrix * vec4( objectTangent, 0.0 ) ).xyz;
	#endif
#endif`,Rw=`float specularStrength;
#ifdef USE_SPECULARMAP
	vec4 texelSpecular = texture2D( specularMap, vSpecularMapUv );
	specularStrength = texelSpecular.r;
#else
	specularStrength = 1.0;
#endif`,Cw=`#ifdef USE_SPECULARMAP
	uniform sampler2D specularMap;
#endif`,bw=`#if defined( TONE_MAPPING )
	gl_FragColor.rgb = toneMapping( gl_FragColor.rgb );
#endif`,Pw=`#ifndef saturate
#define saturate( a ) clamp( a, 0.0, 1.0 )
#endif
uniform float toneMappingExposure;
vec3 LinearToneMapping( vec3 color ) {
	return saturate( toneMappingExposure * color );
}
vec3 ReinhardToneMapping( vec3 color ) {
	color *= toneMappingExposure;
	return saturate( color / ( vec3( 1.0 ) + color ) );
}
vec3 CineonToneMapping( vec3 color ) {
	color *= toneMappingExposure;
	color = max( vec3( 0.0 ), color - 0.004 );
	return pow( ( color * ( 6.2 * color + 0.5 ) ) / ( color * ( 6.2 * color + 1.7 ) + 0.06 ), vec3( 2.2 ) );
}
vec3 RRTAndODTFit( vec3 v ) {
	vec3 a = v * ( v + 0.0245786 ) - 0.000090537;
	vec3 b = v * ( 0.983729 * v + 0.4329510 ) + 0.238081;
	return a / b;
}
vec3 ACESFilmicToneMapping( vec3 color ) {
	const mat3 ACESInputMat = mat3(
		vec3( 0.59719, 0.07600, 0.02840 ),		vec3( 0.35458, 0.90834, 0.13383 ),
		vec3( 0.04823, 0.01566, 0.83777 )
	);
	const mat3 ACESOutputMat = mat3(
		vec3(  1.60475, -0.10208, -0.00327 ),		vec3( -0.53108,  1.10813, -0.07276 ),
		vec3( -0.07367, -0.00605,  1.07602 )
	);
	color *= toneMappingExposure / 0.6;
	color = ACESInputMat * color;
	color = RRTAndODTFit( color );
	color = ACESOutputMat * color;
	return saturate( color );
}
const mat3 LINEAR_REC2020_TO_LINEAR_SRGB = mat3(
	vec3( 1.6605, - 0.1246, - 0.0182 ),
	vec3( - 0.5876, 1.1329, - 0.1006 ),
	vec3( - 0.0728, - 0.0083, 1.1187 )
);
const mat3 LINEAR_SRGB_TO_LINEAR_REC2020 = mat3(
	vec3( 0.6274, 0.0691, 0.0164 ),
	vec3( 0.3293, 0.9195, 0.0880 ),
	vec3( 0.0433, 0.0113, 0.8956 )
);
vec3 agxDefaultContrastApprox( vec3 x ) {
	vec3 x2 = x * x;
	vec3 x4 = x2 * x2;
	return + 15.5 * x4 * x2
		- 40.14 * x4 * x
		+ 31.96 * x4
		- 6.868 * x2 * x
		+ 0.4298 * x2
		+ 0.1191 * x
		- 0.00232;
}
vec3 AgXToneMapping( vec3 color ) {
	const mat3 AgXInsetMatrix = mat3(
		vec3( 0.856627153315983, 0.137318972929847, 0.11189821299995 ),
		vec3( 0.0951212405381588, 0.761241990602591, 0.0767994186031903 ),
		vec3( 0.0482516061458583, 0.101439036467562, 0.811302368396859 )
	);
	const mat3 AgXOutsetMatrix = mat3(
		vec3( 1.1271005818144368, - 0.1413297634984383, - 0.14132976349843826 ),
		vec3( - 0.11060664309660323, 1.157823702216272, - 0.11060664309660294 ),
		vec3( - 0.016493938717834573, - 0.016493938717834257, 1.2519364065950405 )
	);
	const float AgxMinEv = - 12.47393;	const float AgxMaxEv = 4.026069;
	color *= toneMappingExposure;
	color = LINEAR_SRGB_TO_LINEAR_REC2020 * color;
	color = AgXInsetMatrix * color;
	color = max( color, 1e-10 );	color = log2( color );
	color = ( color - AgxMinEv ) / ( AgxMaxEv - AgxMinEv );
	color = clamp( color, 0.0, 1.0 );
	color = agxDefaultContrastApprox( color );
	color = AgXOutsetMatrix * color;
	color = pow( max( vec3( 0.0 ), color ), vec3( 2.2 ) );
	color = LINEAR_REC2020_TO_LINEAR_SRGB * color;
	color = clamp( color, 0.0, 1.0 );
	return color;
}
vec3 NeutralToneMapping( vec3 color ) {
	const float StartCompression = 0.8 - 0.04;
	const float Desaturation = 0.15;
	color *= toneMappingExposure;
	float x = min( color.r, min( color.g, color.b ) );
	float offset = x < 0.08 ? x - 6.25 * x * x : 0.04;
	color -= offset;
	float peak = max( color.r, max( color.g, color.b ) );
	if ( peak < StartCompression ) return color;
	float d = 1. - StartCompression;
	float newPeak = 1. - d * d / ( peak + d - StartCompression );
	color *= newPeak / peak;
	float g = 1. - 1. / ( Desaturation * ( peak - newPeak ) + 1. );
	return mix( color, vec3( newPeak ), g );
}
vec3 CustomToneMapping( vec3 color ) { return color; }`,Lw=`#ifdef USE_TRANSMISSION
	material.transmission = transmission;
	material.transmissionAlpha = 1.0;
	material.thickness = thickness;
	material.attenuationDistance = attenuationDistance;
	material.attenuationColor = attenuationColor;
	#ifdef USE_TRANSMISSIONMAP
		material.transmission *= texture2D( transmissionMap, vTransmissionMapUv ).r;
	#endif
	#ifdef USE_THICKNESSMAP
		material.thickness *= texture2D( thicknessMap, vThicknessMapUv ).g;
	#endif
	vec3 pos = vWorldPosition;
	vec3 v = normalize( cameraPosition - pos );
	vec3 n = inverseTransformDirection( normal, viewMatrix );
	vec4 transmitted = getIBLVolumeRefraction(
		n, v, material.roughness, material.diffuseColor, material.specularColor, material.specularF90,
		pos, modelMatrix, viewMatrix, projectionMatrix, material.dispersion, material.ior, material.thickness,
		material.attenuationColor, material.attenuationDistance );
	material.transmissionAlpha = mix( material.transmissionAlpha, transmitted.a, material.transmission );
	totalDiffuse = mix( totalDiffuse, transmitted.rgb, material.transmission );
#endif`,Iw=`#ifdef USE_TRANSMISSION
	uniform float transmission;
	uniform float thickness;
	uniform float attenuationDistance;
	uniform vec3 attenuationColor;
	#ifdef USE_TRANSMISSIONMAP
		uniform sampler2D transmissionMap;
	#endif
	#ifdef USE_THICKNESSMAP
		uniform sampler2D thicknessMap;
	#endif
	uniform vec2 transmissionSamplerSize;
	uniform sampler2D transmissionSamplerMap;
	uniform mat4 modelMatrix;
	uniform mat4 projectionMatrix;
	varying vec3 vWorldPosition;
	float w0( float a ) {
		return ( 1.0 / 6.0 ) * ( a * ( a * ( - a + 3.0 ) - 3.0 ) + 1.0 );
	}
	float w1( float a ) {
		return ( 1.0 / 6.0 ) * ( a *  a * ( 3.0 * a - 6.0 ) + 4.0 );
	}
	float w2( float a ){
		return ( 1.0 / 6.0 ) * ( a * ( a * ( - 3.0 * a + 3.0 ) + 3.0 ) + 1.0 );
	}
	float w3( float a ) {
		return ( 1.0 / 6.0 ) * ( a * a * a );
	}
	float g0( float a ) {
		return w0( a ) + w1( a );
	}
	float g1( float a ) {
		return w2( a ) + w3( a );
	}
	float h0( float a ) {
		return - 1.0 + w1( a ) / ( w0( a ) + w1( a ) );
	}
	float h1( float a ) {
		return 1.0 + w3( a ) / ( w2( a ) + w3( a ) );
	}
	vec4 bicubic( sampler2D tex, vec2 uv, vec4 texelSize, float lod ) {
		uv = uv * texelSize.zw + 0.5;
		vec2 iuv = floor( uv );
		vec2 fuv = fract( uv );
		float g0x = g0( fuv.x );
		float g1x = g1( fuv.x );
		float h0x = h0( fuv.x );
		float h1x = h1( fuv.x );
		float h0y = h0( fuv.y );
		float h1y = h1( fuv.y );
		vec2 p0 = ( vec2( iuv.x + h0x, iuv.y + h0y ) - 0.5 ) * texelSize.xy;
		vec2 p1 = ( vec2( iuv.x + h1x, iuv.y + h0y ) - 0.5 ) * texelSize.xy;
		vec2 p2 = ( vec2( iuv.x + h0x, iuv.y + h1y ) - 0.5 ) * texelSize.xy;
		vec2 p3 = ( vec2( iuv.x + h1x, iuv.y + h1y ) - 0.5 ) * texelSize.xy;
		return g0( fuv.y ) * ( g0x * textureLod( tex, p0, lod ) + g1x * textureLod( tex, p1, lod ) ) +
			g1( fuv.y ) * ( g0x * textureLod( tex, p2, lod ) + g1x * textureLod( tex, p3, lod ) );
	}
	vec4 textureBicubic( sampler2D sampler, vec2 uv, float lod ) {
		vec2 fLodSize = vec2( textureSize( sampler, int( lod ) ) );
		vec2 cLodSize = vec2( textureSize( sampler, int( lod + 1.0 ) ) );
		vec2 fLodSizeInv = 1.0 / fLodSize;
		vec2 cLodSizeInv = 1.0 / cLodSize;
		vec4 fSample = bicubic( sampler, uv, vec4( fLodSizeInv, fLodSize ), floor( lod ) );
		vec4 cSample = bicubic( sampler, uv, vec4( cLodSizeInv, cLodSize ), ceil( lod ) );
		return mix( fSample, cSample, fract( lod ) );
	}
	vec3 getVolumeTransmissionRay( const in vec3 n, const in vec3 v, const in float thickness, const in float ior, const in mat4 modelMatrix ) {
		vec3 refractionVector = refract( - v, normalize( n ), 1.0 / ior );
		vec3 modelScale;
		modelScale.x = length( vec3( modelMatrix[ 0 ].xyz ) );
		modelScale.y = length( vec3( modelMatrix[ 1 ].xyz ) );
		modelScale.z = length( vec3( modelMatrix[ 2 ].xyz ) );
		return normalize( refractionVector ) * thickness * modelScale;
	}
	float applyIorToRoughness( const in float roughness, const in float ior ) {
		return roughness * clamp( ior * 2.0 - 2.0, 0.0, 1.0 );
	}
	vec4 getTransmissionSample( const in vec2 fragCoord, const in float roughness, const in float ior ) {
		float lod = log2( transmissionSamplerSize.x ) * applyIorToRoughness( roughness, ior );
		return textureBicubic( transmissionSamplerMap, fragCoord.xy, lod );
	}
	vec3 volumeAttenuation( const in float transmissionDistance, const in vec3 attenuationColor, const in float attenuationDistance ) {
		if ( isinf( attenuationDistance ) ) {
			return vec3( 1.0 );
		} else {
			vec3 attenuationCoefficient = -log( attenuationColor ) / attenuationDistance;
			vec3 transmittance = exp( - attenuationCoefficient * transmissionDistance );			return transmittance;
		}
	}
	vec4 getIBLVolumeRefraction( const in vec3 n, const in vec3 v, const in float roughness, const in vec3 diffuseColor,
		const in vec3 specularColor, const in float specularF90, const in vec3 position, const in mat4 modelMatrix,
		const in mat4 viewMatrix, const in mat4 projMatrix, const in float dispersion, const in float ior, const in float thickness,
		const in vec3 attenuationColor, const in float attenuationDistance ) {
		vec4 transmittedLight;
		vec3 transmittance;
		#ifdef USE_DISPERSION
			float halfSpread = ( ior - 1.0 ) * 0.025 * dispersion;
			vec3 iors = vec3( ior - halfSpread, ior, ior + halfSpread );
			for ( int i = 0; i < 3; i ++ ) {
				vec3 transmissionRay = getVolumeTransmissionRay( n, v, thickness, iors[ i ], modelMatrix );
				vec3 refractedRayExit = position + transmissionRay;
		
				vec4 ndcPos = projMatrix * viewMatrix * vec4( refractedRayExit, 1.0 );
				vec2 refractionCoords = ndcPos.xy / ndcPos.w;
				refractionCoords += 1.0;
				refractionCoords /= 2.0;
		
				vec4 transmissionSample = getTransmissionSample( refractionCoords, roughness, iors[ i ] );
				transmittedLight[ i ] = transmissionSample[ i ];
				transmittedLight.a += transmissionSample.a;
				transmittance[ i ] = diffuseColor[ i ] * volumeAttenuation( length( transmissionRay ), attenuationColor, attenuationDistance )[ i ];
			}
			transmittedLight.a /= 3.0;
		
		#else
		
			vec3 transmissionRay = getVolumeTransmissionRay( n, v, thickness, ior, modelMatrix );
			vec3 refractedRayExit = position + transmissionRay;
			vec4 ndcPos = projMatrix * viewMatrix * vec4( refractedRayExit, 1.0 );
			vec2 refractionCoords = ndcPos.xy / ndcPos.w;
			refractionCoords += 1.0;
			refractionCoords /= 2.0;
			transmittedLight = getTransmissionSample( refractionCoords, roughness, ior );
			transmittance = diffuseColor * volumeAttenuation( length( transmissionRay ), attenuationColor, attenuationDistance );
		
		#endif
		vec3 attenuatedColor = transmittance * transmittedLight.rgb;
		vec3 F = EnvironmentBRDF( n, v, specularColor, specularF90, roughness );
		float transmittanceFactor = ( transmittance.r + transmittance.g + transmittance.b ) / 3.0;
		return vec4( ( 1.0 - F ) * attenuatedColor, 1.0 - ( 1.0 - transmittedLight.a ) * transmittanceFactor );
	}
#endif`,Nw=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	varying vec2 vUv;
#endif
#ifdef USE_MAP
	varying vec2 vMapUv;
#endif
#ifdef USE_ALPHAMAP
	varying vec2 vAlphaMapUv;
#endif
#ifdef USE_LIGHTMAP
	varying vec2 vLightMapUv;
#endif
#ifdef USE_AOMAP
	varying vec2 vAoMapUv;
#endif
#ifdef USE_BUMPMAP
	varying vec2 vBumpMapUv;
#endif
#ifdef USE_NORMALMAP
	varying vec2 vNormalMapUv;
#endif
#ifdef USE_EMISSIVEMAP
	varying vec2 vEmissiveMapUv;
#endif
#ifdef USE_METALNESSMAP
	varying vec2 vMetalnessMapUv;
#endif
#ifdef USE_ROUGHNESSMAP
	varying vec2 vRoughnessMapUv;
#endif
#ifdef USE_ANISOTROPYMAP
	varying vec2 vAnisotropyMapUv;
#endif
#ifdef USE_CLEARCOATMAP
	varying vec2 vClearcoatMapUv;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	varying vec2 vClearcoatNormalMapUv;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	varying vec2 vClearcoatRoughnessMapUv;
#endif
#ifdef USE_IRIDESCENCEMAP
	varying vec2 vIridescenceMapUv;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	varying vec2 vIridescenceThicknessMapUv;
#endif
#ifdef USE_SHEEN_COLORMAP
	varying vec2 vSheenColorMapUv;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	varying vec2 vSheenRoughnessMapUv;
#endif
#ifdef USE_SPECULARMAP
	varying vec2 vSpecularMapUv;
#endif
#ifdef USE_SPECULAR_COLORMAP
	varying vec2 vSpecularColorMapUv;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	varying vec2 vSpecularIntensityMapUv;
#endif
#ifdef USE_TRANSMISSIONMAP
	uniform mat3 transmissionMapTransform;
	varying vec2 vTransmissionMapUv;
#endif
#ifdef USE_THICKNESSMAP
	uniform mat3 thicknessMapTransform;
	varying vec2 vThicknessMapUv;
#endif`,Dw=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	varying vec2 vUv;
#endif
#ifdef USE_MAP
	uniform mat3 mapTransform;
	varying vec2 vMapUv;
#endif
#ifdef USE_ALPHAMAP
	uniform mat3 alphaMapTransform;
	varying vec2 vAlphaMapUv;
#endif
#ifdef USE_LIGHTMAP
	uniform mat3 lightMapTransform;
	varying vec2 vLightMapUv;
#endif
#ifdef USE_AOMAP
	uniform mat3 aoMapTransform;
	varying vec2 vAoMapUv;
#endif
#ifdef USE_BUMPMAP
	uniform mat3 bumpMapTransform;
	varying vec2 vBumpMapUv;
#endif
#ifdef USE_NORMALMAP
	uniform mat3 normalMapTransform;
	varying vec2 vNormalMapUv;
#endif
#ifdef USE_DISPLACEMENTMAP
	uniform mat3 displacementMapTransform;
	varying vec2 vDisplacementMapUv;
#endif
#ifdef USE_EMISSIVEMAP
	uniform mat3 emissiveMapTransform;
	varying vec2 vEmissiveMapUv;
#endif
#ifdef USE_METALNESSMAP
	uniform mat3 metalnessMapTransform;
	varying vec2 vMetalnessMapUv;
#endif
#ifdef USE_ROUGHNESSMAP
	uniform mat3 roughnessMapTransform;
	varying vec2 vRoughnessMapUv;
#endif
#ifdef USE_ANISOTROPYMAP
	uniform mat3 anisotropyMapTransform;
	varying vec2 vAnisotropyMapUv;
#endif
#ifdef USE_CLEARCOATMAP
	uniform mat3 clearcoatMapTransform;
	varying vec2 vClearcoatMapUv;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	uniform mat3 clearcoatNormalMapTransform;
	varying vec2 vClearcoatNormalMapUv;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	uniform mat3 clearcoatRoughnessMapTransform;
	varying vec2 vClearcoatRoughnessMapUv;
#endif
#ifdef USE_SHEEN_COLORMAP
	uniform mat3 sheenColorMapTransform;
	varying vec2 vSheenColorMapUv;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	uniform mat3 sheenRoughnessMapTransform;
	varying vec2 vSheenRoughnessMapUv;
#endif
#ifdef USE_IRIDESCENCEMAP
	uniform mat3 iridescenceMapTransform;
	varying vec2 vIridescenceMapUv;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	uniform mat3 iridescenceThicknessMapTransform;
	varying vec2 vIridescenceThicknessMapUv;
#endif
#ifdef USE_SPECULARMAP
	uniform mat3 specularMapTransform;
	varying vec2 vSpecularMapUv;
#endif
#ifdef USE_SPECULAR_COLORMAP
	uniform mat3 specularColorMapTransform;
	varying vec2 vSpecularColorMapUv;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	uniform mat3 specularIntensityMapTransform;
	varying vec2 vSpecularIntensityMapUv;
#endif
#ifdef USE_TRANSMISSIONMAP
	uniform mat3 transmissionMapTransform;
	varying vec2 vTransmissionMapUv;
#endif
#ifdef USE_THICKNESSMAP
	uniform mat3 thicknessMapTransform;
	varying vec2 vThicknessMapUv;
#endif`,Uw=`#if defined( USE_UV ) || defined( USE_ANISOTROPY )
	vUv = vec3( uv, 1 ).xy;
#endif
#ifdef USE_MAP
	vMapUv = ( mapTransform * vec3( MAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ALPHAMAP
	vAlphaMapUv = ( alphaMapTransform * vec3( ALPHAMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_LIGHTMAP
	vLightMapUv = ( lightMapTransform * vec3( LIGHTMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_AOMAP
	vAoMapUv = ( aoMapTransform * vec3( AOMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_BUMPMAP
	vBumpMapUv = ( bumpMapTransform * vec3( BUMPMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_NORMALMAP
	vNormalMapUv = ( normalMapTransform * vec3( NORMALMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_DISPLACEMENTMAP
	vDisplacementMapUv = ( displacementMapTransform * vec3( DISPLACEMENTMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_EMISSIVEMAP
	vEmissiveMapUv = ( emissiveMapTransform * vec3( EMISSIVEMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_METALNESSMAP
	vMetalnessMapUv = ( metalnessMapTransform * vec3( METALNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ROUGHNESSMAP
	vRoughnessMapUv = ( roughnessMapTransform * vec3( ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_ANISOTROPYMAP
	vAnisotropyMapUv = ( anisotropyMapTransform * vec3( ANISOTROPYMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOATMAP
	vClearcoatMapUv = ( clearcoatMapTransform * vec3( CLEARCOATMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOAT_NORMALMAP
	vClearcoatNormalMapUv = ( clearcoatNormalMapTransform * vec3( CLEARCOAT_NORMALMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_CLEARCOAT_ROUGHNESSMAP
	vClearcoatRoughnessMapUv = ( clearcoatRoughnessMapTransform * vec3( CLEARCOAT_ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_IRIDESCENCEMAP
	vIridescenceMapUv = ( iridescenceMapTransform * vec3( IRIDESCENCEMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_IRIDESCENCE_THICKNESSMAP
	vIridescenceThicknessMapUv = ( iridescenceThicknessMapTransform * vec3( IRIDESCENCE_THICKNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SHEEN_COLORMAP
	vSheenColorMapUv = ( sheenColorMapTransform * vec3( SHEEN_COLORMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SHEEN_ROUGHNESSMAP
	vSheenRoughnessMapUv = ( sheenRoughnessMapTransform * vec3( SHEEN_ROUGHNESSMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULARMAP
	vSpecularMapUv = ( specularMapTransform * vec3( SPECULARMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULAR_COLORMAP
	vSpecularColorMapUv = ( specularColorMapTransform * vec3( SPECULAR_COLORMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_SPECULAR_INTENSITYMAP
	vSpecularIntensityMapUv = ( specularIntensityMapTransform * vec3( SPECULAR_INTENSITYMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_TRANSMISSIONMAP
	vTransmissionMapUv = ( transmissionMapTransform * vec3( TRANSMISSIONMAP_UV, 1 ) ).xy;
#endif
#ifdef USE_THICKNESSMAP
	vThicknessMapUv = ( thicknessMapTransform * vec3( THICKNESSMAP_UV, 1 ) ).xy;
#endif`,Fw=`#if defined( USE_ENVMAP ) || defined( DISTANCE ) || defined ( USE_SHADOWMAP ) || defined ( USE_TRANSMISSION ) || NUM_SPOT_LIGHT_COORDS > 0
	vec4 worldPosition = vec4( transformed, 1.0 );
	#ifdef USE_BATCHING
		worldPosition = batchingMatrix * worldPosition;
	#endif
	#ifdef USE_INSTANCING
		worldPosition = instanceMatrix * worldPosition;
	#endif
	worldPosition = modelMatrix * worldPosition;
#endif`;const Ow=`varying vec2 vUv;
uniform mat3 uvTransform;
void main() {
	vUv = ( uvTransform * vec3( uv, 1 ) ).xy;
	gl_Position = vec4( position.xy, 1.0, 1.0 );
}`,kw=`uniform sampler2D t2D;
uniform float backgroundIntensity;
varying vec2 vUv;
void main() {
	vec4 texColor = texture2D( t2D, vUv );
	#ifdef DECODE_VIDEO_TEXTURE
		texColor = vec4( mix( pow( texColor.rgb * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) ), texColor.rgb * 0.0773993808, vec3( lessThanEqual( texColor.rgb, vec3( 0.04045 ) ) ) ), texColor.w );
	#endif
	texColor.rgb *= backgroundIntensity;
	gl_FragColor = texColor;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,Bw=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
	gl_Position.z = gl_Position.w;
}`,zw=`#ifdef ENVMAP_TYPE_CUBE
	uniform samplerCube envMap;
#elif defined( ENVMAP_TYPE_CUBE_UV )
	uniform sampler2D envMap;
#endif
uniform float flipEnvMap;
uniform float backgroundBlurriness;
uniform float backgroundIntensity;
uniform mat3 backgroundRotation;
varying vec3 vWorldDirection;
#include <cube_uv_reflection_fragment>
void main() {
	#ifdef ENVMAP_TYPE_CUBE
		vec4 texColor = textureCube( envMap, backgroundRotation * vec3( flipEnvMap * vWorldDirection.x, vWorldDirection.yz ) );
	#elif defined( ENVMAP_TYPE_CUBE_UV )
		vec4 texColor = textureCubeUV( envMap, backgroundRotation * vWorldDirection, backgroundBlurriness );
	#else
		vec4 texColor = vec4( 0.0, 0.0, 0.0, 1.0 );
	#endif
	texColor.rgb *= backgroundIntensity;
	gl_FragColor = texColor;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,Hw=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
	gl_Position.z = gl_Position.w;
}`,Vw=`uniform samplerCube tCube;
uniform float tFlip;
uniform float opacity;
varying vec3 vWorldDirection;
void main() {
	vec4 texColor = textureCube( tCube, vec3( tFlip * vWorldDirection.x, vWorldDirection.yz ) );
	gl_FragColor = texColor;
	gl_FragColor.a *= opacity;
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,Gw=`#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
varying vec2 vHighPrecisionZW;
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <skinbase_vertex>
	#include <morphinstance_vertex>
	#ifdef USE_DISPLACEMENTMAP
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vHighPrecisionZW = gl_Position.zw;
}`,Ww=`#if DEPTH_PACKING == 3200
	uniform float opacity;
#endif
#include <common>
#include <packing>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
varying vec2 vHighPrecisionZW;
void main() {
	vec4 diffuseColor = vec4( 1.0 );
	#include <clipping_planes_fragment>
	#if DEPTH_PACKING == 3200
		diffuseColor.a = opacity;
	#endif
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <logdepthbuf_fragment>
	float fragCoordZ = 0.5 * vHighPrecisionZW[0] / vHighPrecisionZW[1] + 0.5;
	#if DEPTH_PACKING == 3200
		gl_FragColor = vec4( vec3( 1.0 - fragCoordZ ), opacity );
	#elif DEPTH_PACKING == 3201
		gl_FragColor = packDepthToRGBA( fragCoordZ );
	#elif DEPTH_PACKING == 3202
		gl_FragColor = vec4( packDepthToRGB( fragCoordZ ), 1.0 );
	#elif DEPTH_PACKING == 3203
		gl_FragColor = vec4( packDepthToRG( fragCoordZ ), 0.0, 1.0 );
	#endif
}`,Xw=`#define DISTANCE
varying vec3 vWorldPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <skinbase_vertex>
	#include <morphinstance_vertex>
	#ifdef USE_DISPLACEMENTMAP
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <worldpos_vertex>
	#include <clipping_planes_vertex>
	vWorldPosition = worldPosition.xyz;
}`,jw=`#define DISTANCE
uniform vec3 referencePosition;
uniform float nearDistance;
uniform float farDistance;
varying vec3 vWorldPosition;
#include <common>
#include <packing>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <clipping_planes_pars_fragment>
void main () {
	vec4 diffuseColor = vec4( 1.0 );
	#include <clipping_planes_fragment>
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	float dist = length( vWorldPosition - referencePosition );
	dist = ( dist - nearDistance ) / ( farDistance - nearDistance );
	dist = saturate( dist );
	gl_FragColor = packDepthToRGBA( dist );
}`,Yw=`varying vec3 vWorldDirection;
#include <common>
void main() {
	vWorldDirection = transformDirection( position, modelMatrix );
	#include <begin_vertex>
	#include <project_vertex>
}`,Kw=`uniform sampler2D tEquirect;
varying vec3 vWorldDirection;
#include <common>
void main() {
	vec3 direction = normalize( vWorldDirection );
	vec2 sampleUV = equirectUv( direction );
	gl_FragColor = texture2D( tEquirect, sampleUV );
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
}`,qw=`uniform float scale;
attribute float lineDistance;
varying float vLineDistance;
#include <common>
#include <uv_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	vLineDistance = scale * lineDistance;
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
}`,$w=`uniform vec3 diffuse;
uniform float opacity;
uniform float dashSize;
uniform float totalSize;
varying float vLineDistance;
#include <common>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	if ( mod( vLineDistance, totalSize ) > dashSize ) {
		discard;
	}
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
}`,Zw=`#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#if defined ( USE_ENVMAP ) || defined ( USE_SKINNING )
		#include <beginnormal_vertex>
		#include <morphnormal_vertex>
		#include <skinbase_vertex>
		#include <skinnormal_vertex>
		#include <defaultnormal_vertex>
	#endif
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <fog_vertex>
}`,Qw=`uniform vec3 diffuse;
uniform float opacity;
#ifndef FLAT_SHADED
	varying vec3 vNormal;
#endif
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <fog_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	#ifdef USE_LIGHTMAP
		vec4 lightMapTexel = texture2D( lightMap, vLightMapUv );
		reflectedLight.indirectDiffuse += lightMapTexel.rgb * lightMapIntensity * RECIPROCAL_PI;
	#else
		reflectedLight.indirectDiffuse += vec3( 1.0 );
	#endif
	#include <aomap_fragment>
	reflectedLight.indirectDiffuse *= diffuseColor.rgb;
	vec3 outgoingLight = reflectedLight.indirectDiffuse;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,Jw=`#define LAMBERT
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,eA=`#define LAMBERT
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float opacity;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_lambert_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_lambert_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + totalEmissiveRadiance;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,tA=`#define MATCAP
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <color_pars_vertex>
#include <displacementmap_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
	vViewPosition = - mvPosition.xyz;
}`,nA=`#define MATCAP
uniform vec3 diffuse;
uniform float opacity;
uniform sampler2D matcap;
varying vec3 vViewPosition;
#include <common>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <normal_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	vec3 viewDir = normalize( vViewPosition );
	vec3 x = normalize( vec3( viewDir.z, 0.0, - viewDir.x ) );
	vec3 y = cross( viewDir, x );
	vec2 uv = vec2( dot( x, normal ), dot( y, normal ) ) * 0.495 + 0.5;
	#ifdef USE_MATCAP
		vec4 matcapColor = texture2D( matcap, uv );
	#else
		vec4 matcapColor = vec4( vec3( mix( 0.2, 0.8, uv.y ) ), 1.0 );
	#endif
	vec3 outgoingLight = diffuseColor.rgb * matcapColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,iA=`#define NORMAL
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	varying vec3 vViewPosition;
#endif
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	vViewPosition = - mvPosition.xyz;
#endif
}`,rA=`#define NORMAL
uniform float opacity;
#if defined( FLAT_SHADED ) || defined( USE_BUMPMAP ) || defined( USE_NORMALMAP_TANGENTSPACE )
	varying vec3 vViewPosition;
#endif
#include <packing>
#include <uv_pars_fragment>
#include <normal_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( 0.0, 0.0, 0.0, opacity );
	#include <clipping_planes_fragment>
	#include <logdepthbuf_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	gl_FragColor = vec4( packNormalToRGB( normal ), diffuseColor.a );
	#ifdef OPAQUE
		gl_FragColor.a = 1.0;
	#endif
}`,sA=`#define PHONG
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <envmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <envmap_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,oA=`#define PHONG
uniform vec3 diffuse;
uniform vec3 emissive;
uniform vec3 specular;
uniform float shininess;
uniform float opacity;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_phong_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <specularmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <specularmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_phong_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + reflectedLight.directSpecular + reflectedLight.indirectSpecular + totalEmissiveRadiance;
	#include <envmap_fragment>
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,aA=`#define STANDARD
varying vec3 vViewPosition;
#ifdef USE_TRANSMISSION
	varying vec3 vWorldPosition;
#endif
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
#ifdef USE_TRANSMISSION
	vWorldPosition = worldPosition.xyz;
#endif
}`,lA=`#define STANDARD
#ifdef PHYSICAL
	#define IOR
	#define USE_SPECULAR
#endif
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float roughness;
uniform float metalness;
uniform float opacity;
#ifdef IOR
	uniform float ior;
#endif
#ifdef USE_SPECULAR
	uniform float specularIntensity;
	uniform vec3 specularColor;
	#ifdef USE_SPECULAR_COLORMAP
		uniform sampler2D specularColorMap;
	#endif
	#ifdef USE_SPECULAR_INTENSITYMAP
		uniform sampler2D specularIntensityMap;
	#endif
#endif
#ifdef USE_CLEARCOAT
	uniform float clearcoat;
	uniform float clearcoatRoughness;
#endif
#ifdef USE_DISPERSION
	uniform float dispersion;
#endif
#ifdef USE_IRIDESCENCE
	uniform float iridescence;
	uniform float iridescenceIOR;
	uniform float iridescenceThicknessMinimum;
	uniform float iridescenceThicknessMaximum;
#endif
#ifdef USE_SHEEN
	uniform vec3 sheenColor;
	uniform float sheenRoughness;
	#ifdef USE_SHEEN_COLORMAP
		uniform sampler2D sheenColorMap;
	#endif
	#ifdef USE_SHEEN_ROUGHNESSMAP
		uniform sampler2D sheenRoughnessMap;
	#endif
#endif
#ifdef USE_ANISOTROPY
	uniform vec2 anisotropyVector;
	#ifdef USE_ANISOTROPYMAP
		uniform sampler2D anisotropyMap;
	#endif
#endif
varying vec3 vViewPosition;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <iridescence_fragment>
#include <cube_uv_reflection_fragment>
#include <envmap_common_pars_fragment>
#include <envmap_physical_pars_fragment>
#include <fog_pars_fragment>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_physical_pars_fragment>
#include <transmission_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <clearcoat_pars_fragment>
#include <iridescence_pars_fragment>
#include <roughnessmap_pars_fragment>
#include <metalnessmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <roughnessmap_fragment>
	#include <metalnessmap_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <clearcoat_normal_fragment_begin>
	#include <clearcoat_normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_physical_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 totalDiffuse = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse;
	vec3 totalSpecular = reflectedLight.directSpecular + reflectedLight.indirectSpecular;
	#include <transmission_fragment>
	vec3 outgoingLight = totalDiffuse + totalSpecular + totalEmissiveRadiance;
	#ifdef USE_SHEEN
		float sheenEnergyComp = 1.0 - 0.157 * max3( material.sheenColor );
		outgoingLight = outgoingLight * sheenEnergyComp + sheenSpecularDirect + sheenSpecularIndirect;
	#endif
	#ifdef USE_CLEARCOAT
		float dotNVcc = saturate( dot( geometryClearcoatNormal, geometryViewDir ) );
		vec3 Fcc = F_Schlick( material.clearcoatF0, material.clearcoatF90, dotNVcc );
		outgoingLight = outgoingLight * ( 1.0 - material.clearcoat * Fcc ) + ( clearcoatSpecularDirect + clearcoatSpecularIndirect ) * material.clearcoat;
	#endif
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,cA=`#define TOON
varying vec3 vViewPosition;
#include <common>
#include <batching_pars_vertex>
#include <uv_pars_vertex>
#include <displacementmap_pars_vertex>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <normal_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <shadowmap_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <normal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <displacementmap_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	vViewPosition = - mvPosition.xyz;
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,uA=`#define TOON
uniform vec3 diffuse;
uniform vec3 emissive;
uniform float opacity;
#include <common>
#include <packing>
#include <dithering_pars_fragment>
#include <color_pars_fragment>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <aomap_pars_fragment>
#include <lightmap_pars_fragment>
#include <emissivemap_pars_fragment>
#include <gradientmap_pars_fragment>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <normal_pars_fragment>
#include <lights_toon_pars_fragment>
#include <shadowmap_pars_fragment>
#include <bumpmap_pars_fragment>
#include <normalmap_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	ReflectedLight reflectedLight = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );
	vec3 totalEmissiveRadiance = emissive;
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <color_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	#include <normal_fragment_begin>
	#include <normal_fragment_maps>
	#include <emissivemap_fragment>
	#include <lights_toon_fragment>
	#include <lights_fragment_begin>
	#include <lights_fragment_maps>
	#include <lights_fragment_end>
	#include <aomap_fragment>
	vec3 outgoingLight = reflectedLight.directDiffuse + reflectedLight.indirectDiffuse + totalEmissiveRadiance;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
	#include <dithering_fragment>
}`,dA=`uniform float size;
uniform float scale;
#include <common>
#include <color_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
#ifdef USE_POINTS_UV
	varying vec2 vUv;
	uniform mat3 uvTransform;
#endif
void main() {
	#ifdef USE_POINTS_UV
		vUv = ( uvTransform * vec3( uv, 1 ) ).xy;
	#endif
	#include <color_vertex>
	#include <morphinstance_vertex>
	#include <morphcolor_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <project_vertex>
	gl_PointSize = size;
	#ifdef USE_SIZEATTENUATION
		bool isPerspective = isPerspectiveMatrix( projectionMatrix );
		if ( isPerspective ) gl_PointSize *= ( scale / - mvPosition.z );
	#endif
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <worldpos_vertex>
	#include <fog_vertex>
}`,fA=`uniform vec3 diffuse;
uniform float opacity;
#include <common>
#include <color_pars_fragment>
#include <map_particle_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_particle_fragment>
	#include <color_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
	#include <premultiplied_alpha_fragment>
}`,hA=`#include <common>
#include <batching_pars_vertex>
#include <fog_pars_vertex>
#include <morphtarget_pars_vertex>
#include <skinning_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <shadowmap_pars_vertex>
void main() {
	#include <batching_vertex>
	#include <beginnormal_vertex>
	#include <morphinstance_vertex>
	#include <morphnormal_vertex>
	#include <skinbase_vertex>
	#include <skinnormal_vertex>
	#include <defaultnormal_vertex>
	#include <begin_vertex>
	#include <morphtarget_vertex>
	#include <skinning_vertex>
	#include <project_vertex>
	#include <logdepthbuf_vertex>
	#include <worldpos_vertex>
	#include <shadowmap_vertex>
	#include <fog_vertex>
}`,pA=`uniform vec3 color;
uniform float opacity;
#include <common>
#include <packing>
#include <fog_pars_fragment>
#include <bsdfs>
#include <lights_pars_begin>
#include <logdepthbuf_pars_fragment>
#include <shadowmap_pars_fragment>
#include <shadowmask_pars_fragment>
void main() {
	#include <logdepthbuf_fragment>
	gl_FragColor = vec4( color, opacity * ( 1.0 - getShadowMask() ) );
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
}`,mA=`uniform float rotation;
uniform vec2 center;
#include <common>
#include <uv_pars_vertex>
#include <fog_pars_vertex>
#include <logdepthbuf_pars_vertex>
#include <clipping_planes_pars_vertex>
void main() {
	#include <uv_vertex>
	vec4 mvPosition = modelViewMatrix * vec4( 0.0, 0.0, 0.0, 1.0 );
	vec2 scale;
	scale.x = length( vec3( modelMatrix[ 0 ].x, modelMatrix[ 0 ].y, modelMatrix[ 0 ].z ) );
	scale.y = length( vec3( modelMatrix[ 1 ].x, modelMatrix[ 1 ].y, modelMatrix[ 1 ].z ) );
	#ifndef USE_SIZEATTENUATION
		bool isPerspective = isPerspectiveMatrix( projectionMatrix );
		if ( isPerspective ) scale *= - mvPosition.z;
	#endif
	vec2 alignedPosition = ( position.xy - ( center - vec2( 0.5 ) ) ) * scale;
	vec2 rotatedPosition;
	rotatedPosition.x = cos( rotation ) * alignedPosition.x - sin( rotation ) * alignedPosition.y;
	rotatedPosition.y = sin( rotation ) * alignedPosition.x + cos( rotation ) * alignedPosition.y;
	mvPosition.xy += rotatedPosition;
	gl_Position = projectionMatrix * mvPosition;
	#include <logdepthbuf_vertex>
	#include <clipping_planes_vertex>
	#include <fog_vertex>
}`,gA=`uniform vec3 diffuse;
uniform float opacity;
#include <common>
#include <uv_pars_fragment>
#include <map_pars_fragment>
#include <alphamap_pars_fragment>
#include <alphatest_pars_fragment>
#include <alphahash_pars_fragment>
#include <fog_pars_fragment>
#include <logdepthbuf_pars_fragment>
#include <clipping_planes_pars_fragment>
void main() {
	vec4 diffuseColor = vec4( diffuse, opacity );
	#include <clipping_planes_fragment>
	vec3 outgoingLight = vec3( 0.0 );
	#include <logdepthbuf_fragment>
	#include <map_fragment>
	#include <alphamap_fragment>
	#include <alphatest_fragment>
	#include <alphahash_fragment>
	outgoingLight = diffuseColor.rgb;
	#include <opaque_fragment>
	#include <tonemapping_fragment>
	#include <colorspace_fragment>
	#include <fog_fragment>
}`,Be={alphahash_fragment:kE,alphahash_pars_fragment:BE,alphamap_fragment:zE,alphamap_pars_fragment:HE,alphatest_fragment:VE,alphatest_pars_fragment:GE,aomap_fragment:WE,aomap_pars_fragment:XE,batching_pars_vertex:jE,batching_vertex:YE,begin_vertex:KE,beginnormal_vertex:qE,bsdfs:$E,iridescence_fragment:ZE,bumpmap_pars_fragment:QE,clipping_planes_fragment:JE,clipping_planes_pars_fragment:eT,clipping_planes_pars_vertex:tT,clipping_planes_vertex:nT,color_fragment:iT,color_pars_fragment:rT,color_pars_vertex:sT,color_vertex:oT,common:aT,cube_uv_reflection_fragment:lT,defaultnormal_vertex:cT,displacementmap_pars_vertex:uT,displacementmap_vertex:dT,emissivemap_fragment:fT,emissivemap_pars_fragment:hT,colorspace_fragment:pT,colorspace_pars_fragment:mT,envmap_fragment:gT,envmap_common_pars_fragment:_T,envmap_pars_fragment:vT,envmap_pars_vertex:yT,envmap_physical_pars_fragment:PT,envmap_vertex:xT,fog_vertex:ST,fog_pars_vertex:MT,fog_fragment:ET,fog_pars_fragment:TT,gradientmap_pars_fragment:wT,lightmap_pars_fragment:AT,lights_lambert_fragment:RT,lights_lambert_pars_fragment:CT,lights_pars_begin:bT,lights_toon_fragment:LT,lights_toon_pars_fragment:IT,lights_phong_fragment:NT,lights_phong_pars_fragment:DT,lights_physical_fragment:UT,lights_physical_pars_fragment:FT,lights_fragment_begin:OT,lights_fragment_maps:kT,lights_fragment_end:BT,logdepthbuf_fragment:zT,logdepthbuf_pars_fragment:HT,logdepthbuf_pars_vertex:VT,logdepthbuf_vertex:GT,map_fragment:WT,map_pars_fragment:XT,map_particle_fragment:jT,map_particle_pars_fragment:YT,metalnessmap_fragment:KT,metalnessmap_pars_fragment:qT,morphinstance_vertex:$T,morphcolor_vertex:ZT,morphnormal_vertex:QT,morphtarget_pars_vertex:JT,morphtarget_vertex:ew,normal_fragment_begin:tw,normal_fragment_maps:nw,normal_pars_fragment:iw,normal_pars_vertex:rw,normal_vertex:sw,normalmap_pars_fragment:ow,clearcoat_normal_fragment_begin:aw,clearcoat_normal_fragment_maps:lw,clearcoat_pars_fragment:cw,iridescence_pars_fragment:uw,opaque_fragment:dw,packing:fw,premultiplied_alpha_fragment:hw,project_vertex:pw,dithering_fragment:mw,dithering_pars_fragment:gw,roughnessmap_fragment:_w,roughnessmap_pars_fragment:vw,shadowmap_pars_fragment:yw,shadowmap_pars_vertex:xw,shadowmap_vertex:Sw,shadowmask_pars_fragment:Mw,skinbase_vertex:Ew,skinning_pars_vertex:Tw,skinning_vertex:ww,skinnormal_vertex:Aw,specularmap_fragment:Rw,specularmap_pars_fragment:Cw,tonemapping_fragment:bw,tonemapping_pars_fragment:Pw,transmission_fragment:Lw,transmission_pars_fragment:Iw,uv_pars_fragment:Nw,uv_pars_vertex:Dw,uv_vertex:Uw,worldpos_vertex:Fw,background_vert:Ow,background_frag:kw,backgroundCube_vert:Bw,backgroundCube_frag:zw,cube_vert:Hw,cube_frag:Vw,depth_vert:Gw,depth_frag:Ww,distanceRGBA_vert:Xw,distanceRGBA_frag:jw,equirect_vert:Yw,equirect_frag:Kw,linedashed_vert:qw,linedashed_frag:$w,meshbasic_vert:Zw,meshbasic_frag:Qw,meshlambert_vert:Jw,meshlambert_frag:eA,meshmatcap_vert:tA,meshmatcap_frag:nA,meshnormal_vert:iA,meshnormal_frag:rA,meshphong_vert:sA,meshphong_frag:oA,meshphysical_vert:aA,meshphysical_frag:lA,meshtoon_vert:cA,meshtoon_frag:uA,points_vert:dA,points_frag:fA,shadow_vert:hA,shadow_frag:pA,sprite_vert:mA,sprite_frag:gA},le={common:{diffuse:{value:new Pe(16777215)},opacity:{value:1},map:{value:null},mapTransform:{value:new ze},alphaMap:{value:null},alphaMapTransform:{value:new ze},alphaTest:{value:0}},specularmap:{specularMap:{value:null},specularMapTransform:{value:new ze}},envmap:{envMap:{value:null},envMapRotation:{value:new ze},flipEnvMap:{value:-1},reflectivity:{value:1},ior:{value:1.5},refractionRatio:{value:.98}},aomap:{aoMap:{value:null},aoMapIntensity:{value:1},aoMapTransform:{value:new ze}},lightmap:{lightMap:{value:null},lightMapIntensity:{value:1},lightMapTransform:{value:new ze}},bumpmap:{bumpMap:{value:null},bumpMapTransform:{value:new ze},bumpScale:{value:1}},normalmap:{normalMap:{value:null},normalMapTransform:{value:new ze},normalScale:{value:new we(1,1)}},displacementmap:{displacementMap:{value:null},displacementMapTransform:{value:new ze},displacementScale:{value:1},displacementBias:{value:0}},emissivemap:{emissiveMap:{value:null},emissiveMapTransform:{value:new ze}},metalnessmap:{metalnessMap:{value:null},metalnessMapTransform:{value:new ze}},roughnessmap:{roughnessMap:{value:null},roughnessMapTransform:{value:new ze}},gradientmap:{gradientMap:{value:null}},fog:{fogDensity:{value:25e-5},fogNear:{value:1},fogFar:{value:2e3},fogColor:{value:new Pe(16777215)}},lights:{ambientLightColor:{value:[]},lightProbe:{value:[]},directionalLights:{value:[],properties:{direction:{},color:{}}},directionalLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{}}},directionalShadowMap:{value:[]},directionalShadowMatrix:{value:[]},spotLights:{value:[],properties:{color:{},position:{},direction:{},distance:{},coneCos:{},penumbraCos:{},decay:{}}},spotLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{}}},spotLightMap:{value:[]},spotShadowMap:{value:[]},spotLightMatrix:{value:[]},pointLights:{value:[],properties:{color:{},position:{},decay:{},distance:{}}},pointLightShadows:{value:[],properties:{shadowIntensity:1,shadowBias:{},shadowNormalBias:{},shadowRadius:{},shadowMapSize:{},shadowCameraNear:{},shadowCameraFar:{}}},pointShadowMap:{value:[]},pointShadowMatrix:{value:[]},hemisphereLights:{value:[],properties:{direction:{},skyColor:{},groundColor:{}}},rectAreaLights:{value:[],properties:{color:{},position:{},width:{},height:{}}},ltc_1:{value:null},ltc_2:{value:null}},points:{diffuse:{value:new Pe(16777215)},opacity:{value:1},size:{value:1},scale:{value:1},map:{value:null},alphaMap:{value:null},alphaMapTransform:{value:new ze},alphaTest:{value:0},uvTransform:{value:new ze}},sprite:{diffuse:{value:new Pe(16777215)},opacity:{value:1},center:{value:new we(.5,.5)},rotation:{value:0},map:{value:null},mapTransform:{value:new ze},alphaMap:{value:null},alphaMapTransform:{value:new ze},alphaTest:{value:0}}},ei={basic:{uniforms:$t([le.common,le.specularmap,le.envmap,le.aomap,le.lightmap,le.fog]),vertexShader:Be.meshbasic_vert,fragmentShader:Be.meshbasic_frag},lambert:{uniforms:$t([le.common,le.specularmap,le.envmap,le.aomap,le.lightmap,le.emissivemap,le.bumpmap,le.normalmap,le.displacementmap,le.fog,le.lights,{emissive:{value:new Pe(0)}}]),vertexShader:Be.meshlambert_vert,fragmentShader:Be.meshlambert_frag},phong:{uniforms:$t([le.common,le.specularmap,le.envmap,le.aomap,le.lightmap,le.emissivemap,le.bumpmap,le.normalmap,le.displacementmap,le.fog,le.lights,{emissive:{value:new Pe(0)},specular:{value:new Pe(1118481)},shininess:{value:30}}]),vertexShader:Be.meshphong_vert,fragmentShader:Be.meshphong_frag},standard:{uniforms:$t([le.common,le.envmap,le.aomap,le.lightmap,le.emissivemap,le.bumpmap,le.normalmap,le.displacementmap,le.roughnessmap,le.metalnessmap,le.fog,le.lights,{emissive:{value:new Pe(0)},roughness:{value:1},metalness:{value:0},envMapIntensity:{value:1}}]),vertexShader:Be.meshphysical_vert,fragmentShader:Be.meshphysical_frag},toon:{uniforms:$t([le.common,le.aomap,le.lightmap,le.emissivemap,le.bumpmap,le.normalmap,le.displacementmap,le.gradientmap,le.fog,le.lights,{emissive:{value:new Pe(0)}}]),vertexShader:Be.meshtoon_vert,fragmentShader:Be.meshtoon_frag},matcap:{uniforms:$t([le.common,le.bumpmap,le.normalmap,le.displacementmap,le.fog,{matcap:{value:null}}]),vertexShader:Be.meshmatcap_vert,fragmentShader:Be.meshmatcap_frag},points:{uniforms:$t([le.points,le.fog]),vertexShader:Be.points_vert,fragmentShader:Be.points_frag},dashed:{uniforms:$t([le.common,le.fog,{scale:{value:1},dashSize:{value:1},totalSize:{value:2}}]),vertexShader:Be.linedashed_vert,fragmentShader:Be.linedashed_frag},depth:{uniforms:$t([le.common,le.displacementmap]),vertexShader:Be.depth_vert,fragmentShader:Be.depth_frag},normal:{uniforms:$t([le.common,le.bumpmap,le.normalmap,le.displacementmap,{opacity:{value:1}}]),vertexShader:Be.meshnormal_vert,fragmentShader:Be.meshnormal_frag},sprite:{uniforms:$t([le.sprite,le.fog]),vertexShader:Be.sprite_vert,fragmentShader:Be.sprite_frag},background:{uniforms:{uvTransform:{value:new ze},t2D:{value:null},backgroundIntensity:{value:1}},vertexShader:Be.background_vert,fragmentShader:Be.background_frag},backgroundCube:{uniforms:{envMap:{value:null},flipEnvMap:{value:-1},backgroundBlurriness:{value:0},backgroundIntensity:{value:1},backgroundRotation:{value:new ze}},vertexShader:Be.backgroundCube_vert,fragmentShader:Be.backgroundCube_frag},cube:{uniforms:{tCube:{value:null},tFlip:{value:-1},opacity:{value:1}},vertexShader:Be.cube_vert,fragmentShader:Be.cube_frag},equirect:{uniforms:{tEquirect:{value:null}},vertexShader:Be.equirect_vert,fragmentShader:Be.equirect_frag},distanceRGBA:{uniforms:$t([le.common,le.displacementmap,{referencePosition:{value:new L},nearDistance:{value:1},farDistance:{value:1e3}}]),vertexShader:Be.distanceRGBA_vert,fragmentShader:Be.distanceRGBA_frag},shadow:{uniforms:$t([le.lights,le.fog,{color:{value:new Pe(0)},opacity:{value:1}}]),vertexShader:Be.shadow_vert,fragmentShader:Be.shadow_frag}};ei.physical={uniforms:$t([ei.standard.uniforms,{clearcoat:{value:0},clearcoatMap:{value:null},clearcoatMapTransform:{value:new ze},clearcoatNormalMap:{value:null},clearcoatNormalMapTransform:{value:new ze},clearcoatNormalScale:{value:new we(1,1)},clearcoatRoughness:{value:0},clearcoatRoughnessMap:{value:null},clearcoatRoughnessMapTransform:{value:new ze},dispersion:{value:0},iridescence:{value:0},iridescenceMap:{value:null},iridescenceMapTransform:{value:new ze},iridescenceIOR:{value:1.3},iridescenceThicknessMinimum:{value:100},iridescenceThicknessMaximum:{value:400},iridescenceThicknessMap:{value:null},iridescenceThicknessMapTransform:{value:new ze},sheen:{value:0},sheenColor:{value:new Pe(0)},sheenColorMap:{value:null},sheenColorMapTransform:{value:new ze},sheenRoughness:{value:1},sheenRoughnessMap:{value:null},sheenRoughnessMapTransform:{value:new ze},transmission:{value:0},transmissionMap:{value:null},transmissionMapTransform:{value:new ze},transmissionSamplerSize:{value:new we},transmissionSamplerMap:{value:null},thickness:{value:0},thicknessMap:{value:null},thicknessMapTransform:{value:new ze},attenuationDistance:{value:0},attenuationColor:{value:new Pe(0)},specularColor:{value:new Pe(1,1,1)},specularColorMap:{value:null},specularColorMapTransform:{value:new ze},specularIntensity:{value:1},specularIntensityMap:{value:null},specularIntensityMapTransform:{value:new ze},anisotropyVector:{value:new we},anisotropyMap:{value:null},anisotropyMapTransform:{value:new ze}}]),vertexShader:Be.meshphysical_vert,fragmentShader:Be.meshphysical_frag};const dl={r:0,b:0,g:0},Mr=new ai,_A=new Ue;function vA(n,e,t,i,r,s,o){const a=new Pe(0);let l=s===!0?0:1,c,u,d=null,f=0,p=null;function g(_){let v=_.isScene===!0?_.background:null;return v&&v.isTexture&&(v=(_.backgroundBlurriness>0?t:e).get(v)),v}function y(_){let v=!1;const x=g(_);x===null?h(a,l):x&&x.isColor&&(h(x,1),v=!0);const R=n.xr.getEnvironmentBlendMode();R==="additive"?i.buffers.color.setClear(0,0,0,1,o):R==="alpha-blend"&&i.buffers.color.setClear(0,0,0,0,o),(n.autoClear||v)&&(i.buffers.depth.setTest(!0),i.buffers.depth.setMask(!0),i.buffers.color.setMask(!0),n.clear(n.autoClearColor,n.autoClearDepth,n.autoClearStencil))}function m(_,v){const x=g(v);x&&(x.isCubeTexture||x.mapping===zc)?(u===void 0&&(u=new tn(new Aa(1,1,1),new Di({name:"BackgroundCubeMaterial",uniforms:Qs(ei.backgroundCube.uniforms),vertexShader:ei.backgroundCube.vertexShader,fragmentShader:ei.backgroundCube.fragmentShader,side:fn,depthTest:!1,depthWrite:!1,fog:!1})),u.geometry.deleteAttribute("normal"),u.geometry.deleteAttribute("uv"),u.onBeforeRender=function(R,A,w){this.matrixWorld.copyPosition(w.matrixWorld)},Object.defineProperty(u.material,"envMap",{get:function(){return this.uniforms.envMap.value}}),r.update(u)),Mr.copy(v.backgroundRotation),Mr.x*=-1,Mr.y*=-1,Mr.z*=-1,x.isCubeTexture&&x.isRenderTargetTexture===!1&&(Mr.y*=-1,Mr.z*=-1),u.material.uniforms.envMap.value=x,u.material.uniforms.flipEnvMap.value=x.isCubeTexture&&x.isRenderTargetTexture===!1?-1:1,u.material.uniforms.backgroundBlurriness.value=v.backgroundBlurriness,u.material.uniforms.backgroundIntensity.value=v.backgroundIntensity,u.material.uniforms.backgroundRotation.value.setFromMatrix4(_A.makeRotationFromEuler(Mr)),u.material.toneMapped=qe.getTransfer(x.colorSpace)!==lt,(d!==x||f!==x.version||p!==n.toneMapping)&&(u.material.needsUpdate=!0,d=x,f=x.version,p=n.toneMapping),u.layers.enableAll(),_.unshift(u,u.geometry,u.material,0,0,null)):x&&x.isTexture&&(c===void 0&&(c=new tn(new Vc(2,2),new Di({name:"BackgroundMaterial",uniforms:Qs(ei.background.uniforms),vertexShader:ei.background.vertexShader,fragmentShader:ei.background.fragmentShader,side:Ii,depthTest:!1,depthWrite:!1,fog:!1})),c.geometry.deleteAttribute("normal"),Object.defineProperty(c.material,"map",{get:function(){return this.uniforms.t2D.value}}),r.update(c)),c.material.uniforms.t2D.value=x,c.material.uniforms.backgroundIntensity.value=v.backgroundIntensity,c.material.toneMapped=qe.getTransfer(x.colorSpace)!==lt,x.matrixAutoUpdate===!0&&x.updateMatrix(),c.material.uniforms.uvTransform.value.copy(x.matrix),(d!==x||f!==x.version||p!==n.toneMapping)&&(c.material.needsUpdate=!0,d=x,f=x.version,p=n.toneMapping),c.layers.enableAll(),_.unshift(c,c.geometry,c.material,0,0,null))}function h(_,v){_.getRGB(dl,Bv(n)),i.buffers.color.setClear(dl.r,dl.g,dl.b,v,o)}return{getClearColor:function(){return a},setClearColor:function(_,v=1){a.set(_),l=v,h(a,l)},getClearAlpha:function(){return l},setClearAlpha:function(_){l=_,h(a,l)},render:y,addToRenderList:m}}function yA(n,e){const t=n.getParameter(n.MAX_VERTEX_ATTRIBS),i={},r=f(null);let s=r,o=!1;function a(M,I,V,B,j){let Y=!1;const W=d(B,V,I);s!==W&&(s=W,c(s.object)),Y=p(M,B,V,j),Y&&g(M,B,V,j),j!==null&&e.update(j,n.ELEMENT_ARRAY_BUFFER),(Y||o)&&(o=!1,x(M,I,V,B),j!==null&&n.bindBuffer(n.ELEMENT_ARRAY_BUFFER,e.get(j).buffer))}function l(){return n.createVertexArray()}function c(M){return n.bindVertexArray(M)}function u(M){return n.deleteVertexArray(M)}function d(M,I,V){const B=V.wireframe===!0;let j=i[M.id];j===void 0&&(j={},i[M.id]=j);let Y=j[I.id];Y===void 0&&(Y={},j[I.id]=Y);let W=Y[B];return W===void 0&&(W=f(l()),Y[B]=W),W}function f(M){const I=[],V=[],B=[];for(let j=0;j<t;j++)I[j]=0,V[j]=0,B[j]=0;return{geometry:null,program:null,wireframe:!1,newAttributes:I,enabledAttributes:V,attributeDivisors:B,object:M,attributes:{},index:null}}function p(M,I,V,B){const j=s.attributes,Y=I.attributes;let W=0;const $=V.getAttributes();for(const N in $)if($[N].location>=0){const Q=j[N];let oe=Y[N];if(oe===void 0&&(N==="instanceMatrix"&&M.instanceMatrix&&(oe=M.instanceMatrix),N==="instanceColor"&&M.instanceColor&&(oe=M.instanceColor)),Q===void 0||Q.attribute!==oe||oe&&Q.data!==oe.data)return!0;W++}return s.attributesNum!==W||s.index!==B}function g(M,I,V,B){const j={},Y=I.attributes;let W=0;const $=V.getAttributes();for(const N in $)if($[N].location>=0){let Q=Y[N];Q===void 0&&(N==="instanceMatrix"&&M.instanceMatrix&&(Q=M.instanceMatrix),N==="instanceColor"&&M.instanceColor&&(Q=M.instanceColor));const oe={};oe.attribute=Q,Q&&Q.data&&(oe.data=Q.data),j[N]=oe,W++}s.attributes=j,s.attributesNum=W,s.index=B}function y(){const M=s.newAttributes;for(let I=0,V=M.length;I<V;I++)M[I]=0}function m(M){h(M,0)}function h(M,I){const V=s.newAttributes,B=s.enabledAttributes,j=s.attributeDivisors;V[M]=1,B[M]===0&&(n.enableVertexAttribArray(M),B[M]=1),j[M]!==I&&(n.vertexAttribDivisor(M,I),j[M]=I)}function _(){const M=s.newAttributes,I=s.enabledAttributes;for(let V=0,B=I.length;V<B;V++)I[V]!==M[V]&&(n.disableVertexAttribArray(V),I[V]=0)}function v(M,I,V,B,j,Y,W){W===!0?n.vertexAttribIPointer(M,I,V,j,Y):n.vertexAttribPointer(M,I,V,B,j,Y)}function x(M,I,V,B){y();const j=B.attributes,Y=V.getAttributes(),W=I.defaultAttributeValues;for(const $ in Y){const N=Y[$];if(N.location>=0){let K=j[$];if(K===void 0&&($==="instanceMatrix"&&M.instanceMatrix&&(K=M.instanceMatrix),$==="instanceColor"&&M.instanceColor&&(K=M.instanceColor)),K!==void 0){const Q=K.normalized,oe=K.itemSize,Me=e.get(K);if(Me===void 0)continue;const je=Me.buffer,G=Me.type,ne=Me.bytesPerElement,he=G===n.INT||G===n.UNSIGNED_INT||K.gpuType===kh;if(K.isInterleavedBufferAttribute){const de=K.data,Ce=de.stride,Fe=K.offset;if(de.isInstancedInterleavedBuffer){for(let We=0;We<N.locationSize;We++)h(N.location+We,de.meshPerAttribute);M.isInstancedMesh!==!0&&B._maxInstanceCount===void 0&&(B._maxInstanceCount=de.meshPerAttribute*de.count)}else for(let We=0;We<N.locationSize;We++)m(N.location+We);n.bindBuffer(n.ARRAY_BUFFER,je);for(let We=0;We<N.locationSize;We++)v(N.location+We,oe/N.locationSize,G,Q,Ce*ne,(Fe+oe/N.locationSize*We)*ne,he)}else{if(K.isInstancedBufferAttribute){for(let de=0;de<N.locationSize;de++)h(N.location+de,K.meshPerAttribute);M.isInstancedMesh!==!0&&B._maxInstanceCount===void 0&&(B._maxInstanceCount=K.meshPerAttribute*K.count)}else for(let de=0;de<N.locationSize;de++)m(N.location+de);n.bindBuffer(n.ARRAY_BUFFER,je);for(let de=0;de<N.locationSize;de++)v(N.location+de,oe/N.locationSize,G,Q,oe*ne,oe/N.locationSize*de*ne,he)}}else if(W!==void 0){const Q=W[$];if(Q!==void 0)switch(Q.length){case 2:n.vertexAttrib2fv(N.location,Q);break;case 3:n.vertexAttrib3fv(N.location,Q);break;case 4:n.vertexAttrib4fv(N.location,Q);break;default:n.vertexAttrib1fv(N.location,Q)}}}}_()}function R(){b();for(const M in i){const I=i[M];for(const V in I){const B=I[V];for(const j in B)u(B[j].object),delete B[j];delete I[V]}delete i[M]}}function A(M){if(i[M.id]===void 0)return;const I=i[M.id];for(const V in I){const B=I[V];for(const j in B)u(B[j].object),delete B[j];delete I[V]}delete i[M.id]}function w(M){for(const I in i){const V=i[I];if(V[M.id]===void 0)continue;const B=V[M.id];for(const j in B)u(B[j].object),delete B[j];delete V[M.id]}}function b(){T(),o=!0,s!==r&&(s=r,c(s.object))}function T(){r.geometry=null,r.program=null,r.wireframe=!1}return{setup:a,reset:b,resetDefaultState:T,dispose:R,releaseStatesOfGeometry:A,releaseStatesOfProgram:w,initAttributes:y,enableAttribute:m,disableUnusedAttributes:_}}function xA(n,e,t){let i;function r(c){i=c}function s(c,u){n.drawArrays(i,c,u),t.update(u,i,1)}function o(c,u,d){d!==0&&(n.drawArraysInstanced(i,c,u,d),t.update(u,i,d))}function a(c,u,d){if(d===0)return;e.get("WEBGL_multi_draw").multiDrawArraysWEBGL(i,c,0,u,0,d);let p=0;for(let g=0;g<d;g++)p+=u[g];t.update(p,i,1)}function l(c,u,d,f){if(d===0)return;const p=e.get("WEBGL_multi_draw");if(p===null)for(let g=0;g<c.length;g++)o(c[g],u[g],f[g]);else{p.multiDrawArraysInstancedWEBGL(i,c,0,u,0,f,0,d);let g=0;for(let y=0;y<d;y++)g+=u[y];for(let y=0;y<f.length;y++)t.update(g,i,f[y])}}this.setMode=r,this.render=s,this.renderInstances=o,this.renderMultiDraw=a,this.renderMultiDrawInstances=l}function SA(n,e,t,i){let r;function s(){if(r!==void 0)return r;if(e.has("EXT_texture_filter_anisotropic")===!0){const A=e.get("EXT_texture_filter_anisotropic");r=n.getParameter(A.MAX_TEXTURE_MAX_ANISOTROPY_EXT)}else r=0;return r}function o(A){return!(A!==Pn&&i.convert(A)!==n.getParameter(n.IMPLEMENTATION_COLOR_READ_FORMAT))}function a(A){const w=A===Ta&&(e.has("EXT_color_buffer_half_float")||e.has("EXT_color_buffer_float"));return!(A!==Ni&&i.convert(A)!==n.getParameter(n.IMPLEMENTATION_COLOR_READ_TYPE)&&A!==Xn&&!w)}function l(A){if(A==="highp"){if(n.getShaderPrecisionFormat(n.VERTEX_SHADER,n.HIGH_FLOAT).precision>0&&n.getShaderPrecisionFormat(n.FRAGMENT_SHADER,n.HIGH_FLOAT).precision>0)return"highp";A="mediump"}return A==="mediump"&&n.getShaderPrecisionFormat(n.VERTEX_SHADER,n.MEDIUM_FLOAT).precision>0&&n.getShaderPrecisionFormat(n.FRAGMENT_SHADER,n.MEDIUM_FLOAT).precision>0?"mediump":"lowp"}let c=t.precision!==void 0?t.precision:"highp";const u=l(c);u!==c&&(console.warn("THREE.WebGLRenderer:",c,"not supported, using",u,"instead."),c=u);const d=t.logarithmicDepthBuffer===!0,f=n.getParameter(n.MAX_TEXTURE_IMAGE_UNITS),p=n.getParameter(n.MAX_VERTEX_TEXTURE_IMAGE_UNITS),g=n.getParameter(n.MAX_TEXTURE_SIZE),y=n.getParameter(n.MAX_CUBE_MAP_TEXTURE_SIZE),m=n.getParameter(n.MAX_VERTEX_ATTRIBS),h=n.getParameter(n.MAX_VERTEX_UNIFORM_VECTORS),_=n.getParameter(n.MAX_VARYING_VECTORS),v=n.getParameter(n.MAX_FRAGMENT_UNIFORM_VECTORS),x=p>0,R=n.getParameter(n.MAX_SAMPLES);return{isWebGL2:!0,getMaxAnisotropy:s,getMaxPrecision:l,textureFormatReadable:o,textureTypeReadable:a,precision:c,logarithmicDepthBuffer:d,maxTextures:f,maxVertexTextures:p,maxTextureSize:g,maxCubemapSize:y,maxAttributes:m,maxVertexUniforms:h,maxVaryings:_,maxFragmentUniforms:v,vertexTextures:x,maxSamples:R}}function MA(n){const e=this;let t=null,i=0,r=!1,s=!1;const o=new Rr,a=new ze,l={value:null,needsUpdate:!1};this.uniform=l,this.numPlanes=0,this.numIntersection=0,this.init=function(d,f){const p=d.length!==0||f||i!==0||r;return r=f,i=d.length,p},this.beginShadows=function(){s=!0,u(null)},this.endShadows=function(){s=!1},this.setGlobalState=function(d,f){t=u(d,f,0)},this.setState=function(d,f,p){const g=d.clippingPlanes,y=d.clipIntersection,m=d.clipShadows,h=n.get(d);if(!r||g===null||g.length===0||s&&!m)s?u(null):c();else{const _=s?0:i,v=_*4;let x=h.clippingState||null;l.value=x,x=u(g,f,v,p);for(let R=0;R!==v;++R)x[R]=t[R];h.clippingState=x,this.numIntersection=y?this.numPlanes:0,this.numPlanes+=_}};function c(){l.value!==t&&(l.value=t,l.needsUpdate=i>0),e.numPlanes=i,e.numIntersection=0}function u(d,f,p,g){const y=d!==null?d.length:0;let m=null;if(y!==0){if(m=l.value,g!==!0||m===null){const h=p+y*4,_=f.matrixWorldInverse;a.getNormalMatrix(_),(m===null||m.length<h)&&(m=new Float32Array(h));for(let v=0,x=p;v!==y;++v,x+=4)o.copy(d[v]).applyMatrix4(_,a),o.normal.toArray(m,x),m[x+3]=o.constant}l.value=m,l.needsUpdate=!0}return e.numPlanes=y,e.numIntersection=0,m}}function EA(n){let e=new WeakMap;function t(o,a){return a===sf?o.mapping=js:a===of&&(o.mapping=Ys),o}function i(o){if(o&&o.isTexture){const a=o.mapping;if(a===sf||a===of)if(e.has(o)){const l=e.get(o).texture;return t(l,o.mapping)}else{const l=o.image;if(l&&l.height>0){const c=new DE(l.height);return c.fromEquirectangularTexture(n,o),e.set(o,c),o.addEventListener("dispose",r),t(c.texture,o.mapping)}else return null}}return o}function r(o){const a=o.target;a.removeEventListener("dispose",r);const l=e.get(a);l!==void 0&&(e.delete(a),l.dispose())}function s(){e=new WeakMap}return{get:i,dispose:s}}class qh extends zv{constructor(e=-1,t=1,i=1,r=-1,s=.1,o=2e3){super(),this.isOrthographicCamera=!0,this.type="OrthographicCamera",this.zoom=1,this.view=null,this.left=e,this.right=t,this.top=i,this.bottom=r,this.near=s,this.far=o,this.updateProjectionMatrix()}copy(e,t){return super.copy(e,t),this.left=e.left,this.right=e.right,this.top=e.top,this.bottom=e.bottom,this.near=e.near,this.far=e.far,this.zoom=e.zoom,this.view=e.view===null?null:Object.assign({},e.view),this}setViewOffset(e,t,i,r,s,o){this.view===null&&(this.view={enabled:!0,fullWidth:1,fullHeight:1,offsetX:0,offsetY:0,width:1,height:1}),this.view.enabled=!0,this.view.fullWidth=e,this.view.fullHeight=t,this.view.offsetX=i,this.view.offsetY=r,this.view.width=s,this.view.height=o,this.updateProjectionMatrix()}clearViewOffset(){this.view!==null&&(this.view.enabled=!1),this.updateProjectionMatrix()}updateProjectionMatrix(){const e=(this.right-this.left)/(2*this.zoom),t=(this.top-this.bottom)/(2*this.zoom),i=(this.right+this.left)/2,r=(this.top+this.bottom)/2;let s=i-e,o=i+e,a=r+t,l=r-t;if(this.view!==null&&this.view.enabled){const c=(this.right-this.left)/this.view.fullWidth/this.zoom,u=(this.top-this.bottom)/this.view.fullHeight/this.zoom;s+=c*this.view.offsetX,o=s+c*this.view.width,a-=u*this.view.offsetY,l=a-u*this.view.height}this.projectionMatrix.makeOrthographic(s,o,a,l,this.near,this.far,this.coordinateSystem),this.projectionMatrixInverse.copy(this.projectionMatrix).invert()}toJSON(e){const t=super.toJSON(e);return t.object.zoom=this.zoom,t.object.left=this.left,t.object.right=this.right,t.object.top=this.top,t.object.bottom=this.bottom,t.object.near=this.near,t.object.far=this.far,this.view!==null&&(t.object.view=Object.assign({},this.view)),t}}const Rs=4,Qm=[.125,.215,.35,.446,.526,.582],Pr=20,zu=new qh,Jm=new Pe;let Hu=null,Vu=0,Gu=0,Wu=!1;const Cr=(1+Math.sqrt(5))/2,cs=1/Cr,eg=[new L(-Cr,cs,0),new L(Cr,cs,0),new L(-cs,0,Cr),new L(cs,0,Cr),new L(0,Cr,-cs),new L(0,Cr,cs),new L(-1,1,-1),new L(1,1,-1),new L(-1,1,1),new L(1,1,1)];class tg{constructor(e){this._renderer=e,this._pingPongRenderTarget=null,this._lodMax=0,this._cubeSize=0,this._lodPlanes=[],this._sizeLods=[],this._sigmas=[],this._blurMaterial=null,this._cubemapMaterial=null,this._equirectMaterial=null,this._compileMaterial(this._blurMaterial)}fromScene(e,t=0,i=.1,r=100){Hu=this._renderer.getRenderTarget(),Vu=this._renderer.getActiveCubeFace(),Gu=this._renderer.getActiveMipmapLevel(),Wu=this._renderer.xr.enabled,this._renderer.xr.enabled=!1,this._setSize(256);const s=this._allocateTargets();return s.depthBuffer=!0,this._sceneToCubeUV(e,i,r,s),t>0&&this._blur(s,0,0,t),this._applyPMREM(s),this._cleanup(s),s}fromEquirectangular(e,t=null){return this._fromTexture(e,t)}fromCubemap(e,t=null){return this._fromTexture(e,t)}compileCubemapShader(){this._cubemapMaterial===null&&(this._cubemapMaterial=rg(),this._compileMaterial(this._cubemapMaterial))}compileEquirectangularShader(){this._equirectMaterial===null&&(this._equirectMaterial=ig(),this._compileMaterial(this._equirectMaterial))}dispose(){this._dispose(),this._cubemapMaterial!==null&&this._cubemapMaterial.dispose(),this._equirectMaterial!==null&&this._equirectMaterial.dispose()}_setSize(e){this._lodMax=Math.floor(Math.log2(e)),this._cubeSize=Math.pow(2,this._lodMax)}_dispose(){this._blurMaterial!==null&&this._blurMaterial.dispose(),this._pingPongRenderTarget!==null&&this._pingPongRenderTarget.dispose();for(let e=0;e<this._lodPlanes.length;e++)this._lodPlanes[e].dispose()}_cleanup(e){this._renderer.setRenderTarget(Hu,Vu,Gu),this._renderer.xr.enabled=Wu,e.scissorTest=!1,fl(e,0,0,e.width,e.height)}_fromTexture(e,t){e.mapping===js||e.mapping===Ys?this._setSize(e.image.length===0?16:e.image[0].width||e.image[0].image.width):this._setSize(e.image.width/4),Hu=this._renderer.getRenderTarget(),Vu=this._renderer.getActiveCubeFace(),Gu=this._renderer.getActiveMipmapLevel(),Wu=this._renderer.xr.enabled,this._renderer.xr.enabled=!1;const i=t||this._allocateTargets();return this._textureToCubeUV(e,i),this._applyPMREM(i),this._cleanup(i),i}_allocateTargets(){const e=3*Math.max(this._cubeSize,112),t=4*this._cubeSize,i={magFilter:an,minFilter:an,generateMipmaps:!1,type:Ta,format:Pn,colorSpace:Vt,depthBuffer:!1},r=ng(e,t,i);if(this._pingPongRenderTarget===null||this._pingPongRenderTarget.width!==e||this._pingPongRenderTarget.height!==t){this._pingPongRenderTarget!==null&&this._dispose(),this._pingPongRenderTarget=ng(e,t,i);const{_lodMax:s}=this;({sizeLods:this._sizeLods,lodPlanes:this._lodPlanes,sigmas:this._sigmas}=TA(s)),this._blurMaterial=wA(s,e,t)}return r}_compileMaterial(e){const t=new tn(this._lodPlanes[0],e);this._renderer.compile(t,zu)}_sceneToCubeUV(e,t,i,r){const a=new Jt(90,1,t,i),l=[1,-1,1,1,1,1],c=[1,1,1,-1,-1,-1],u=this._renderer,d=u.autoClear,f=u.toneMapping;u.getClearColor(Jm),u.toneMapping=ur,u.autoClear=!1;const p=new Ai({name:"PMREM.Background",side:fn,depthWrite:!1,depthTest:!1}),g=new tn(new Aa,p);let y=!1;const m=e.background;m?m.isColor&&(p.color.copy(m),e.background=null,y=!0):(p.color.copy(Jm),y=!0);for(let h=0;h<6;h++){const _=h%3;_===0?(a.up.set(0,l[h],0),a.lookAt(c[h],0,0)):_===1?(a.up.set(0,0,l[h]),a.lookAt(0,c[h],0)):(a.up.set(0,l[h],0),a.lookAt(0,0,c[h]));const v=this._cubeSize;fl(r,_*v,h>2?v:0,v,v),u.setRenderTarget(r),y&&u.render(g,a),u.render(e,a)}g.geometry.dispose(),g.material.dispose(),u.toneMapping=f,u.autoClear=d,e.background=m}_textureToCubeUV(e,t){const i=this._renderer,r=e.mapping===js||e.mapping===Ys;r?(this._cubemapMaterial===null&&(this._cubemapMaterial=rg()),this._cubemapMaterial.uniforms.flipEnvMap.value=e.isRenderTargetTexture===!1?-1:1):this._equirectMaterial===null&&(this._equirectMaterial=ig());const s=r?this._cubemapMaterial:this._equirectMaterial,o=new tn(this._lodPlanes[0],s),a=s.uniforms;a.envMap.value=e;const l=this._cubeSize;fl(t,0,0,3*l,2*l),i.setRenderTarget(t),i.render(o,zu)}_applyPMREM(e){const t=this._renderer,i=t.autoClear;t.autoClear=!1;const r=this._lodPlanes.length;for(let s=1;s<r;s++){const o=Math.sqrt(this._sigmas[s]*this._sigmas[s]-this._sigmas[s-1]*this._sigmas[s-1]),a=eg[(r-s-1)%eg.length];this._blur(e,s-1,s,o,a)}t.autoClear=i}_blur(e,t,i,r,s){const o=this._pingPongRenderTarget;this._halfBlur(e,o,t,i,r,"latitudinal",s),this._halfBlur(o,e,i,i,r,"longitudinal",s)}_halfBlur(e,t,i,r,s,o,a){const l=this._renderer,c=this._blurMaterial;o!=="latitudinal"&&o!=="longitudinal"&&console.error("blur direction must be either latitudinal or longitudinal!");const u=3,d=new tn(this._lodPlanes[r],c),f=c.uniforms,p=this._sizeLods[i]-1,g=isFinite(s)?Math.PI/(2*p):2*Math.PI/(2*Pr-1),y=s/g,m=isFinite(s)?1+Math.floor(u*y):Pr;m>Pr&&console.warn(`sigmaRadians, ${s}, is too large and will clip, as it requested ${m} samples when the maximum is set to ${Pr}`);const h=[];let _=0;for(let w=0;w<Pr;++w){const b=w/y,T=Math.exp(-b*b/2);h.push(T),w===0?_+=T:w<m&&(_+=2*T)}for(let w=0;w<h.length;w++)h[w]=h[w]/_;f.envMap.value=e.texture,f.samples.value=m,f.weights.value=h,f.latitudinal.value=o==="latitudinal",a&&(f.poleAxis.value=a);const{_lodMax:v}=this;f.dTheta.value=g,f.mipInt.value=v-i;const x=this._sizeLods[r],R=3*x*(r>v-Rs?r-v+Rs:0),A=4*(this._cubeSize-x);fl(t,R,A,3*x,2*x),l.setRenderTarget(t),l.render(d,zu)}}function TA(n){const e=[],t=[],i=[];let r=n;const s=n-Rs+1+Qm.length;for(let o=0;o<s;o++){const a=Math.pow(2,r);t.push(a);let l=1/a;o>n-Rs?l=Qm[o-n+Rs-1]:o===0&&(l=0),i.push(l);const c=1/(a-2),u=-c,d=1+c,f=[u,u,d,u,d,d,u,u,d,d,u,d],p=6,g=6,y=3,m=2,h=1,_=new Float32Array(y*g*p),v=new Float32Array(m*g*p),x=new Float32Array(h*g*p);for(let A=0;A<p;A++){const w=A%3*2/3-1,b=A>2?0:-1,T=[w,b,0,w+2/3,b,0,w+2/3,b+1,0,w,b,0,w+2/3,b+1,0,w,b+1,0];_.set(T,y*g*A),v.set(f,m*g*A);const M=[A,A,A,A,A,A];x.set(M,h*g*A)}const R=new hn;R.setAttribute("position",new Rt(_,y)),R.setAttribute("uv",new Rt(v,m)),R.setAttribute("faceIndex",new Rt(x,h)),e.push(R),r>Rs&&r--}return{lodPlanes:e,sizeLods:t,sigmas:i}}function ng(n,e,t){const i=new Vr(n,e,t);return i.texture.mapping=zc,i.texture.name="PMREM.cubeUv",i.scissorTest=!0,i}function fl(n,e,t,i,r){n.viewport.set(e,t,i,r),n.scissor.set(e,t,i,r)}function wA(n,e,t){const i=new Float32Array(Pr),r=new L(0,1,0);return new Di({name:"SphericalGaussianBlur",defines:{n:Pr,CUBEUV_TEXEL_WIDTH:1/e,CUBEUV_TEXEL_HEIGHT:1/t,CUBEUV_MAX_MIP:`${n}.0`},uniforms:{envMap:{value:null},samples:{value:1},weights:{value:i},latitudinal:{value:!1},dTheta:{value:0},mipInt:{value:0},poleAxis:{value:r}},vertexShader:$h(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			varying vec3 vOutputDirection;

			uniform sampler2D envMap;
			uniform int samples;
			uniform float weights[ n ];
			uniform bool latitudinal;
			uniform float dTheta;
			uniform float mipInt;
			uniform vec3 poleAxis;

			#define ENVMAP_TYPE_CUBE_UV
			#include <cube_uv_reflection_fragment>

			vec3 getSample( float theta, vec3 axis ) {

				float cosTheta = cos( theta );
				// Rodrigues' axis-angle rotation
				vec3 sampleDirection = vOutputDirection * cosTheta
					+ cross( axis, vOutputDirection ) * sin( theta )
					+ axis * dot( axis, vOutputDirection ) * ( 1.0 - cosTheta );

				return bilinearCubeUV( envMap, sampleDirection, mipInt );

			}

			void main() {

				vec3 axis = latitudinal ? poleAxis : cross( poleAxis, vOutputDirection );

				if ( all( equal( axis, vec3( 0.0 ) ) ) ) {

					axis = vec3( vOutputDirection.z, 0.0, - vOutputDirection.x );

				}

				axis = normalize( axis );

				gl_FragColor = vec4( 0.0, 0.0, 0.0, 1.0 );
				gl_FragColor.rgb += weights[ 0 ] * getSample( 0.0, axis );

				for ( int i = 1; i < n; i++ ) {

					if ( i >= samples ) {

						break;

					}

					float theta = dTheta * float( i );
					gl_FragColor.rgb += weights[ i ] * getSample( -1.0 * theta, axis );
					gl_FragColor.rgb += weights[ i ] * getSample( theta, axis );

				}

			}
		`,blending:cr,depthTest:!1,depthWrite:!1})}function ig(){return new Di({name:"EquirectangularToCubeUV",uniforms:{envMap:{value:null}},vertexShader:$h(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			varying vec3 vOutputDirection;

			uniform sampler2D envMap;

			#include <common>

			void main() {

				vec3 outputDirection = normalize( vOutputDirection );
				vec2 uv = equirectUv( outputDirection );

				gl_FragColor = vec4( texture2D ( envMap, uv ).rgb, 1.0 );

			}
		`,blending:cr,depthTest:!1,depthWrite:!1})}function rg(){return new Di({name:"CubemapToCubeUV",uniforms:{envMap:{value:null},flipEnvMap:{value:-1}},vertexShader:$h(),fragmentShader:`

			precision mediump float;
			precision mediump int;

			uniform float flipEnvMap;

			varying vec3 vOutputDirection;

			uniform samplerCube envMap;

			void main() {

				gl_FragColor = textureCube( envMap, vec3( flipEnvMap * vOutputDirection.x, vOutputDirection.yz ) );

			}
		`,blending:cr,depthTest:!1,depthWrite:!1})}function $h(){return`

		precision mediump float;
		precision mediump int;

		attribute float faceIndex;

		varying vec3 vOutputDirection;

		// RH coordinate system; PMREM face-indexing convention
		vec3 getDirection( vec2 uv, float face ) {

			uv = 2.0 * uv - 1.0;

			vec3 direction = vec3( uv, 1.0 );

			if ( face == 0.0 ) {

				direction = direction.zyx; // ( 1, v, u ) pos x

			} else if ( face == 1.0 ) {

				direction = direction.xzy;
				direction.xz *= -1.0; // ( -u, 1, -v ) pos y

			} else if ( face == 2.0 ) {

				direction.x *= -1.0; // ( -u, v, 1 ) pos z

			} else if ( face == 3.0 ) {

				direction = direction.zyx;
				direction.xz *= -1.0; // ( -1, v, -u ) neg x

			} else if ( face == 4.0 ) {

				direction = direction.xzy;
				direction.xy *= -1.0; // ( -u, -1, v ) neg y

			} else if ( face == 5.0 ) {

				direction.z *= -1.0; // ( u, v, -1 ) neg z

			}

			return direction;

		}

		void main() {

			vOutputDirection = getDirection( uv, faceIndex );
			gl_Position = vec4( position, 1.0 );

		}
	`}function AA(n){let e=new WeakMap,t=null;function i(a){if(a&&a.isTexture){const l=a.mapping,c=l===sf||l===of,u=l===js||l===Ys;if(c||u){let d=e.get(a);const f=d!==void 0?d.texture.pmremVersion:0;if(a.isRenderTargetTexture&&a.pmremVersion!==f)return t===null&&(t=new tg(n)),d=c?t.fromEquirectangular(a,d):t.fromCubemap(a,d),d.texture.pmremVersion=a.pmremVersion,e.set(a,d),d.texture;if(d!==void 0)return d.texture;{const p=a.image;return c&&p&&p.height>0||u&&p&&r(p)?(t===null&&(t=new tg(n)),d=c?t.fromEquirectangular(a):t.fromCubemap(a),d.texture.pmremVersion=a.pmremVersion,e.set(a,d),a.addEventListener("dispose",s),d.texture):null}}}return a}function r(a){let l=0;const c=6;for(let u=0;u<c;u++)a[u]!==void 0&&l++;return l===c}function s(a){const l=a.target;l.removeEventListener("dispose",s);const c=e.get(l);c!==void 0&&(e.delete(l),c.dispose())}function o(){e=new WeakMap,t!==null&&(t.dispose(),t=null)}return{get:i,dispose:o}}function RA(n){const e={};function t(i){if(e[i]!==void 0)return e[i];let r;switch(i){case"WEBGL_depth_texture":r=n.getExtension("WEBGL_depth_texture")||n.getExtension("MOZ_WEBGL_depth_texture")||n.getExtension("WEBKIT_WEBGL_depth_texture");break;case"EXT_texture_filter_anisotropic":r=n.getExtension("EXT_texture_filter_anisotropic")||n.getExtension("MOZ_EXT_texture_filter_anisotropic")||n.getExtension("WEBKIT_EXT_texture_filter_anisotropic");break;case"WEBGL_compressed_texture_s3tc":r=n.getExtension("WEBGL_compressed_texture_s3tc")||n.getExtension("MOZ_WEBGL_compressed_texture_s3tc")||n.getExtension("WEBKIT_WEBGL_compressed_texture_s3tc");break;case"WEBGL_compressed_texture_pvrtc":r=n.getExtension("WEBGL_compressed_texture_pvrtc")||n.getExtension("WEBKIT_WEBGL_compressed_texture_pvrtc");break;default:r=n.getExtension(i)}return e[i]=r,r}return{has:function(i){return t(i)!==null},init:function(){t("EXT_color_buffer_float"),t("WEBGL_clip_cull_distance"),t("OES_texture_float_linear"),t("EXT_color_buffer_half_float"),t("WEBGL_multisampled_render_to_texture"),t("WEBGL_render_shared_exponent")},get:function(i){const r=t(i);return r===null&&Fs("THREE.WebGLRenderer: "+i+" extension not supported."),r}}}function CA(n,e,t,i){const r={},s=new WeakMap;function o(d){const f=d.target;f.index!==null&&e.remove(f.index);for(const g in f.attributes)e.remove(f.attributes[g]);for(const g in f.morphAttributes){const y=f.morphAttributes[g];for(let m=0,h=y.length;m<h;m++)e.remove(y[m])}f.removeEventListener("dispose",o),delete r[f.id];const p=s.get(f);p&&(e.remove(p),s.delete(f)),i.releaseStatesOfGeometry(f),f.isInstancedBufferGeometry===!0&&delete f._maxInstanceCount,t.memory.geometries--}function a(d,f){return r[f.id]===!0||(f.addEventListener("dispose",o),r[f.id]=!0,t.memory.geometries++),f}function l(d){const f=d.attributes;for(const g in f)e.update(f[g],n.ARRAY_BUFFER);const p=d.morphAttributes;for(const g in p){const y=p[g];for(let m=0,h=y.length;m<h;m++)e.update(y[m],n.ARRAY_BUFFER)}}function c(d){const f=[],p=d.index,g=d.attributes.position;let y=0;if(p!==null){const _=p.array;y=p.version;for(let v=0,x=_.length;v<x;v+=3){const R=_[v+0],A=_[v+1],w=_[v+2];f.push(R,A,A,w,w,R)}}else if(g!==void 0){const _=g.array;y=g.version;for(let v=0,x=_.length/3-1;v<x;v+=3){const R=v+0,A=v+1,w=v+2;f.push(R,A,A,w,w,R)}}else return;const m=new(Nv(f)?kv:Ov)(f,1);m.version=y;const h=s.get(d);h&&e.remove(h),s.set(d,m)}function u(d){const f=s.get(d);if(f){const p=d.index;p!==null&&f.version<p.version&&c(d)}else c(d);return s.get(d)}return{get:a,update:l,getWireframeAttribute:u}}function bA(n,e,t){let i;function r(f){i=f}let s,o;function a(f){s=f.type,o=f.bytesPerElement}function l(f,p){n.drawElements(i,p,s,f*o),t.update(p,i,1)}function c(f,p,g){g!==0&&(n.drawElementsInstanced(i,p,s,f*o,g),t.update(p,i,g))}function u(f,p,g){if(g===0)return;e.get("WEBGL_multi_draw").multiDrawElementsWEBGL(i,p,0,s,f,0,g);let m=0;for(let h=0;h<g;h++)m+=p[h];t.update(m,i,1)}function d(f,p,g,y){if(g===0)return;const m=e.get("WEBGL_multi_draw");if(m===null)for(let h=0;h<f.length;h++)c(f[h]/o,p[h],y[h]);else{m.multiDrawElementsInstancedWEBGL(i,p,0,s,f,0,y,0,g);let h=0;for(let _=0;_<g;_++)h+=p[_];for(let _=0;_<y.length;_++)t.update(h,i,y[_])}}this.setMode=r,this.setIndex=a,this.render=l,this.renderInstances=c,this.renderMultiDraw=u,this.renderMultiDrawInstances=d}function PA(n){const e={geometries:0,textures:0},t={frame:0,calls:0,triangles:0,points:0,lines:0};function i(s,o,a){switch(t.calls++,o){case n.TRIANGLES:t.triangles+=a*(s/3);break;case n.LINES:t.lines+=a*(s/2);break;case n.LINE_STRIP:t.lines+=a*(s-1);break;case n.LINE_LOOP:t.lines+=a*s;break;case n.POINTS:t.points+=a*s;break;default:console.error("THREE.WebGLInfo: Unknown draw mode:",o);break}}function r(){t.calls=0,t.triangles=0,t.points=0,t.lines=0}return{memory:e,render:t,programs:null,autoReset:!0,reset:r,update:i}}function LA(n,e,t){const i=new WeakMap,r=new st;function s(o,a,l){const c=o.morphTargetInfluences,u=a.morphAttributes.position||a.morphAttributes.normal||a.morphAttributes.color,d=u!==void 0?u.length:0;let f=i.get(a);if(f===void 0||f.count!==d){let M=function(){b.dispose(),i.delete(a),a.removeEventListener("dispose",M)};var p=M;f!==void 0&&f.texture.dispose();const g=a.morphAttributes.position!==void 0,y=a.morphAttributes.normal!==void 0,m=a.morphAttributes.color!==void 0,h=a.morphAttributes.position||[],_=a.morphAttributes.normal||[],v=a.morphAttributes.color||[];let x=0;g===!0&&(x=1),y===!0&&(x=2),m===!0&&(x=3);let R=a.attributes.position.count*x,A=1;R>e.maxTextureSize&&(A=Math.ceil(R/e.maxTextureSize),R=e.maxTextureSize);const w=new Float32Array(R*A*4*d),b=new Uv(w,R,A,d);b.type=Xn,b.needsUpdate=!0;const T=x*4;for(let I=0;I<d;I++){const V=h[I],B=_[I],j=v[I],Y=R*A*4*I;for(let W=0;W<V.count;W++){const $=W*T;g===!0&&(r.fromBufferAttribute(V,W),w[Y+$+0]=r.x,w[Y+$+1]=r.y,w[Y+$+2]=r.z,w[Y+$+3]=0),y===!0&&(r.fromBufferAttribute(B,W),w[Y+$+4]=r.x,w[Y+$+5]=r.y,w[Y+$+6]=r.z,w[Y+$+7]=0),m===!0&&(r.fromBufferAttribute(j,W),w[Y+$+8]=r.x,w[Y+$+9]=r.y,w[Y+$+10]=r.z,w[Y+$+11]=j.itemSize===4?r.w:1)}}f={count:d,texture:b,size:new we(R,A)},i.set(a,f),a.addEventListener("dispose",M)}if(o.isInstancedMesh===!0&&o.morphTexture!==null)l.getUniforms().setValue(n,"morphTexture",o.morphTexture,t);else{let g=0;for(let m=0;m<c.length;m++)g+=c[m];const y=a.morphTargetsRelative?1:1-g;l.getUniforms().setValue(n,"morphTargetBaseInfluence",y),l.getUniforms().setValue(n,"morphTargetInfluences",c)}l.getUniforms().setValue(n,"morphTargetsTexture",f.texture,t),l.getUniforms().setValue(n,"morphTargetsTextureSize",f.size)}return{update:s}}function IA(n,e,t,i){let r=new WeakMap;function s(l){const c=i.render.frame,u=l.geometry,d=e.get(l,u);if(r.get(d)!==c&&(e.update(d),r.set(d,c)),l.isInstancedMesh&&(l.hasEventListener("dispose",a)===!1&&l.addEventListener("dispose",a),r.get(l)!==c&&(t.update(l.instanceMatrix,n.ARRAY_BUFFER),l.instanceColor!==null&&t.update(l.instanceColor,n.ARRAY_BUFFER),r.set(l,c))),l.isSkinnedMesh){const f=l.skeleton;r.get(f)!==c&&(f.update(),r.set(f,c))}return d}function o(){r=new WeakMap}function a(l){const c=l.target;c.removeEventListener("dispose",a),t.remove(c.instanceMatrix),c.instanceColor!==null&&t.remove(c.instanceColor)}return{update:s,dispose:o}}class Gv extends Pt{constructor(e,t,i,r,s,o,a,l,c,u=Us){if(u!==Us&&u!==$s)throw new Error("DepthTexture format must be either THREE.DepthFormat or THREE.DepthStencilFormat");i===void 0&&u===Us&&(i=Hr),i===void 0&&u===$s&&(i=qs),super(null,r,s,o,a,l,u,i,c),this.isDepthTexture=!0,this.image={width:e,height:t},this.magFilter=a!==void 0?a:en,this.minFilter=l!==void 0?l:en,this.flipY=!1,this.generateMipmaps=!1,this.compareFunction=null}copy(e){return super.copy(e),this.compareFunction=e.compareFunction,this}toJSON(e){const t=super.toJSON(e);return this.compareFunction!==null&&(t.compareFunction=this.compareFunction),t}}const Wv=new Pt,sg=new Gv(1,1),Xv=new Uv,jv=new vE,Yv=new Hv,og=[],ag=[],lg=new Float32Array(16),cg=new Float32Array(9),ug=new Float32Array(4);function oo(n,e,t){const i=n[0];if(i<=0||i>0)return n;const r=e*t;let s=og[r];if(s===void 0&&(s=new Float32Array(r),og[r]=s),e!==0){i.toArray(s,0);for(let o=1,a=0;o!==e;++o)a+=t,n[o].toArray(s,a)}return s}function Lt(n,e){if(n.length!==e.length)return!1;for(let t=0,i=n.length;t<i;t++)if(n[t]!==e[t])return!1;return!0}function It(n,e){for(let t=0,i=e.length;t<i;t++)n[t]=e[t]}function Gc(n,e){let t=ag[e];t===void 0&&(t=new Int32Array(e),ag[e]=t);for(let i=0;i!==e;++i)t[i]=n.allocateTextureUnit();return t}function NA(n,e){const t=this.cache;t[0]!==e&&(n.uniform1f(this.addr,e),t[0]=e)}function DA(n,e){const t=this.cache;if(e.x!==void 0)(t[0]!==e.x||t[1]!==e.y)&&(n.uniform2f(this.addr,e.x,e.y),t[0]=e.x,t[1]=e.y);else{if(Lt(t,e))return;n.uniform2fv(this.addr,e),It(t,e)}}function UA(n,e){const t=this.cache;if(e.x!==void 0)(t[0]!==e.x||t[1]!==e.y||t[2]!==e.z)&&(n.uniform3f(this.addr,e.x,e.y,e.z),t[0]=e.x,t[1]=e.y,t[2]=e.z);else if(e.r!==void 0)(t[0]!==e.r||t[1]!==e.g||t[2]!==e.b)&&(n.uniform3f(this.addr,e.r,e.g,e.b),t[0]=e.r,t[1]=e.g,t[2]=e.b);else{if(Lt(t,e))return;n.uniform3fv(this.addr,e),It(t,e)}}function FA(n,e){const t=this.cache;if(e.x!==void 0)(t[0]!==e.x||t[1]!==e.y||t[2]!==e.z||t[3]!==e.w)&&(n.uniform4f(this.addr,e.x,e.y,e.z,e.w),t[0]=e.x,t[1]=e.y,t[2]=e.z,t[3]=e.w);else{if(Lt(t,e))return;n.uniform4fv(this.addr,e),It(t,e)}}function OA(n,e){const t=this.cache,i=e.elements;if(i===void 0){if(Lt(t,e))return;n.uniformMatrix2fv(this.addr,!1,e),It(t,e)}else{if(Lt(t,i))return;ug.set(i),n.uniformMatrix2fv(this.addr,!1,ug),It(t,i)}}function kA(n,e){const t=this.cache,i=e.elements;if(i===void 0){if(Lt(t,e))return;n.uniformMatrix3fv(this.addr,!1,e),It(t,e)}else{if(Lt(t,i))return;cg.set(i),n.uniformMatrix3fv(this.addr,!1,cg),It(t,i)}}function BA(n,e){const t=this.cache,i=e.elements;if(i===void 0){if(Lt(t,e))return;n.uniformMatrix4fv(this.addr,!1,e),It(t,e)}else{if(Lt(t,i))return;lg.set(i),n.uniformMatrix4fv(this.addr,!1,lg),It(t,i)}}function zA(n,e){const t=this.cache;t[0]!==e&&(n.uniform1i(this.addr,e),t[0]=e)}function HA(n,e){const t=this.cache;if(e.x!==void 0)(t[0]!==e.x||t[1]!==e.y)&&(n.uniform2i(this.addr,e.x,e.y),t[0]=e.x,t[1]=e.y);else{if(Lt(t,e))return;n.uniform2iv(this.addr,e),It(t,e)}}function VA(n,e){const t=this.cache;if(e.x!==void 0)(t[0]!==e.x||t[1]!==e.y||t[2]!==e.z)&&(n.uniform3i(this.addr,e.x,e.y,e.z),t[0]=e.x,t[1]=e.y,t[2]=e.z);else{if(Lt(t,e))return;n.uniform3iv(this.addr,e),It(t,e)}}function GA(n,e){const t=this.cache;if(e.x!==void 0)(t[0]!==e.x||t[1]!==e.y||t[2]!==e.z||t[3]!==e.w)&&(n.uniform4i(this.addr,e.x,e.y,e.z,e.w),t[0]=e.x,t[1]=e.y,t[2]=e.z,t[3]=e.w);else{if(Lt(t,e))return;n.uniform4iv(this.addr,e),It(t,e)}}function WA(n,e){const t=this.cache;t[0]!==e&&(n.uniform1ui(this.addr,e),t[0]=e)}function XA(n,e){const t=this.cache;if(e.x!==void 0)(t[0]!==e.x||t[1]!==e.y)&&(n.uniform2ui(this.addr,e.x,e.y),t[0]=e.x,t[1]=e.y);else{if(Lt(t,e))return;n.uniform2uiv(this.addr,e),It(t,e)}}function jA(n,e){const t=this.cache;if(e.x!==void 0)(t[0]!==e.x||t[1]!==e.y||t[2]!==e.z)&&(n.uniform3ui(this.addr,e.x,e.y,e.z),t[0]=e.x,t[1]=e.y,t[2]=e.z);else{if(Lt(t,e))return;n.uniform3uiv(this.addr,e),It(t,e)}}function YA(n,e){const t=this.cache;if(e.x!==void 0)(t[0]!==e.x||t[1]!==e.y||t[2]!==e.z||t[3]!==e.w)&&(n.uniform4ui(this.addr,e.x,e.y,e.z,e.w),t[0]=e.x,t[1]=e.y,t[2]=e.z,t[3]=e.w);else{if(Lt(t,e))return;n.uniform4uiv(this.addr,e),It(t,e)}}function KA(n,e,t){const i=this.cache,r=t.allocateTextureUnit();i[0]!==r&&(n.uniform1i(this.addr,r),i[0]=r);let s;this.type===n.SAMPLER_2D_SHADOW?(sg.compareFunction=Iv,s=sg):s=Wv,t.setTexture2D(e||s,r)}function qA(n,e,t){const i=this.cache,r=t.allocateTextureUnit();i[0]!==r&&(n.uniform1i(this.addr,r),i[0]=r),t.setTexture3D(e||jv,r)}function $A(n,e,t){const i=this.cache,r=t.allocateTextureUnit();i[0]!==r&&(n.uniform1i(this.addr,r),i[0]=r),t.setTextureCube(e||Yv,r)}function ZA(n,e,t){const i=this.cache,r=t.allocateTextureUnit();i[0]!==r&&(n.uniform1i(this.addr,r),i[0]=r),t.setTexture2DArray(e||Xv,r)}function QA(n){switch(n){case 5126:return NA;case 35664:return DA;case 35665:return UA;case 35666:return FA;case 35674:return OA;case 35675:return kA;case 35676:return BA;case 5124:case 35670:return zA;case 35667:case 35671:return HA;case 35668:case 35672:return VA;case 35669:case 35673:return GA;case 5125:return WA;case 36294:return XA;case 36295:return jA;case 36296:return YA;case 35678:case 36198:case 36298:case 36306:case 35682:return KA;case 35679:case 36299:case 36307:return qA;case 35680:case 36300:case 36308:case 36293:return $A;case 36289:case 36303:case 36311:case 36292:return ZA}}function JA(n,e){n.uniform1fv(this.addr,e)}function e1(n,e){const t=oo(e,this.size,2);n.uniform2fv(this.addr,t)}function t1(n,e){const t=oo(e,this.size,3);n.uniform3fv(this.addr,t)}function n1(n,e){const t=oo(e,this.size,4);n.uniform4fv(this.addr,t)}function i1(n,e){const t=oo(e,this.size,4);n.uniformMatrix2fv(this.addr,!1,t)}function r1(n,e){const t=oo(e,this.size,9);n.uniformMatrix3fv(this.addr,!1,t)}function s1(n,e){const t=oo(e,this.size,16);n.uniformMatrix4fv(this.addr,!1,t)}function o1(n,e){n.uniform1iv(this.addr,e)}function a1(n,e){n.uniform2iv(this.addr,e)}function l1(n,e){n.uniform3iv(this.addr,e)}function c1(n,e){n.uniform4iv(this.addr,e)}function u1(n,e){n.uniform1uiv(this.addr,e)}function d1(n,e){n.uniform2uiv(this.addr,e)}function f1(n,e){n.uniform3uiv(this.addr,e)}function h1(n,e){n.uniform4uiv(this.addr,e)}function p1(n,e,t){const i=this.cache,r=e.length,s=Gc(t,r);Lt(i,s)||(n.uniform1iv(this.addr,s),It(i,s));for(let o=0;o!==r;++o)t.setTexture2D(e[o]||Wv,s[o])}function m1(n,e,t){const i=this.cache,r=e.length,s=Gc(t,r);Lt(i,s)||(n.uniform1iv(this.addr,s),It(i,s));for(let o=0;o!==r;++o)t.setTexture3D(e[o]||jv,s[o])}function g1(n,e,t){const i=this.cache,r=e.length,s=Gc(t,r);Lt(i,s)||(n.uniform1iv(this.addr,s),It(i,s));for(let o=0;o!==r;++o)t.setTextureCube(e[o]||Yv,s[o])}function _1(n,e,t){const i=this.cache,r=e.length,s=Gc(t,r);Lt(i,s)||(n.uniform1iv(this.addr,s),It(i,s));for(let o=0;o!==r;++o)t.setTexture2DArray(e[o]||Xv,s[o])}function v1(n){switch(n){case 5126:return JA;case 35664:return e1;case 35665:return t1;case 35666:return n1;case 35674:return i1;case 35675:return r1;case 35676:return s1;case 5124:case 35670:return o1;case 35667:case 35671:return a1;case 35668:case 35672:return l1;case 35669:case 35673:return c1;case 5125:return u1;case 36294:return d1;case 36295:return f1;case 36296:return h1;case 35678:case 36198:case 36298:case 36306:case 35682:return p1;case 35679:case 36299:case 36307:return m1;case 35680:case 36300:case 36308:case 36293:return g1;case 36289:case 36303:case 36311:case 36292:return _1}}class y1{constructor(e,t,i){this.id=e,this.addr=i,this.cache=[],this.type=t.type,this.setValue=QA(t.type)}}class x1{constructor(e,t,i){this.id=e,this.addr=i,this.cache=[],this.type=t.type,this.size=t.size,this.setValue=v1(t.type)}}class S1{constructor(e){this.id=e,this.seq=[],this.map={}}setValue(e,t,i){const r=this.seq;for(let s=0,o=r.length;s!==o;++s){const a=r[s];a.setValue(e,t[a.id],i)}}}const Xu=/(\w+)(\])?(\[|\.)?/g;function dg(n,e){n.seq.push(e),n.map[e.id]=e}function M1(n,e,t){const i=n.name,r=i.length;for(Xu.lastIndex=0;;){const s=Xu.exec(i),o=Xu.lastIndex;let a=s[1];const l=s[2]==="]",c=s[3];if(l&&(a=a|0),c===void 0||c==="["&&o+2===r){dg(t,c===void 0?new y1(a,n,e):new x1(a,n,e));break}else{let d=t.map[a];d===void 0&&(d=new S1(a),dg(t,d)),t=d}}}class Wl{constructor(e,t){this.seq=[],this.map={};const i=e.getProgramParameter(t,e.ACTIVE_UNIFORMS);for(let r=0;r<i;++r){const s=e.getActiveUniform(t,r),o=e.getUniformLocation(t,s.name);M1(s,o,this)}}setValue(e,t,i,r){const s=this.map[t];s!==void 0&&s.setValue(e,i,r)}setOptional(e,t,i){const r=t[i];r!==void 0&&this.setValue(e,i,r)}static upload(e,t,i,r){for(let s=0,o=t.length;s!==o;++s){const a=t[s],l=i[a.id];l.needsUpdate!==!1&&a.setValue(e,l.value,r)}}static seqWithValue(e,t){const i=[];for(let r=0,s=e.length;r!==s;++r){const o=e[r];o.id in t&&i.push(o)}return i}}function fg(n,e,t){const i=n.createShader(e);return n.shaderSource(i,t),n.compileShader(i),i}const E1=37297;let T1=0;function w1(n,e){const t=n.split(`
`),i=[],r=Math.max(e-6,0),s=Math.min(e+6,t.length);for(let o=r;o<s;o++){const a=o+1;i.push(`${a===e?">":" "} ${a}: ${t[o]}`)}return i.join(`
`)}function A1(n){const e=qe.getPrimaries(qe.workingColorSpace),t=qe.getPrimaries(n);let i;switch(e===t?i="":e===xc&&t===yc?i="LinearDisplayP3ToLinearSRGB":e===yc&&t===xc&&(i="LinearSRGBToLinearDisplayP3"),n){case Vt:case Hc:return[i,"LinearTransferOETF"];case Qt:case Xh:return[i,"sRGBTransferOETF"];default:return console.warn("THREE.WebGLProgram: Unsupported color space:",n),[i,"LinearTransferOETF"]}}function hg(n,e,t){const i=n.getShaderParameter(e,n.COMPILE_STATUS),r=n.getShaderInfoLog(e).trim();if(i&&r==="")return"";const s=/ERROR: 0:(\d+)/.exec(r);if(s){const o=parseInt(s[1]);return t.toUpperCase()+`

`+r+`

`+w1(n.getShaderSource(e),o)}else return r}function R1(n,e){const t=A1(e);return`vec4 ${n}( vec4 value ) { return ${t[0]}( ${t[1]}( value ) ); }`}function C1(n,e){let t;switch(e){case CM:t="Linear";break;case bM:t="Reinhard";break;case PM:t="Cineon";break;case LM:t="ACESFilmic";break;case NM:t="AgX";break;case DM:t="Neutral";break;case IM:t="Custom";break;default:console.warn("THREE.WebGLProgram: Unsupported toneMapping:",e),t="Linear"}return"vec3 "+n+"( vec3 color ) { return "+t+"ToneMapping( color ); }"}const hl=new L;function b1(){qe.getLuminanceCoefficients(hl);const n=hl.x.toFixed(4),e=hl.y.toFixed(4),t=hl.z.toFixed(4);return["float luminance( const in vec3 rgb ) {",`	const vec3 weights = vec3( ${n}, ${e}, ${t} );`,"	return dot( weights, rgb );","}"].join(`
`)}function P1(n){return[n.extensionClipCullDistance?"#extension GL_ANGLE_clip_cull_distance : require":"",n.extensionMultiDraw?"#extension GL_ANGLE_multi_draw : require":""].filter(Oo).join(`
`)}function L1(n){const e=[];for(const t in n){const i=n[t];i!==!1&&e.push("#define "+t+" "+i)}return e.join(`
`)}function I1(n,e){const t={},i=n.getProgramParameter(e,n.ACTIVE_ATTRIBUTES);for(let r=0;r<i;r++){const s=n.getActiveAttrib(e,r),o=s.name;let a=1;s.type===n.FLOAT_MAT2&&(a=2),s.type===n.FLOAT_MAT3&&(a=3),s.type===n.FLOAT_MAT4&&(a=4),t[o]={type:s.type,location:n.getAttribLocation(e,o),locationSize:a}}return t}function Oo(n){return n!==""}function pg(n,e){const t=e.numSpotLightShadows+e.numSpotLightMaps-e.numSpotLightShadowsWithMaps;return n.replace(/NUM_DIR_LIGHTS/g,e.numDirLights).replace(/NUM_SPOT_LIGHTS/g,e.numSpotLights).replace(/NUM_SPOT_LIGHT_MAPS/g,e.numSpotLightMaps).replace(/NUM_SPOT_LIGHT_COORDS/g,t).replace(/NUM_RECT_AREA_LIGHTS/g,e.numRectAreaLights).replace(/NUM_POINT_LIGHTS/g,e.numPointLights).replace(/NUM_HEMI_LIGHTS/g,e.numHemiLights).replace(/NUM_DIR_LIGHT_SHADOWS/g,e.numDirLightShadows).replace(/NUM_SPOT_LIGHT_SHADOWS_WITH_MAPS/g,e.numSpotLightShadowsWithMaps).replace(/NUM_SPOT_LIGHT_SHADOWS/g,e.numSpotLightShadows).replace(/NUM_POINT_LIGHT_SHADOWS/g,e.numPointLightShadows)}function mg(n,e){return n.replace(/NUM_CLIPPING_PLANES/g,e.numClippingPlanes).replace(/UNION_CLIPPING_PLANES/g,e.numClippingPlanes-e.numClipIntersection)}const N1=/^[ \t]*#include +<([\w\d./]+)>/gm;function Uf(n){return n.replace(N1,U1)}const D1=new Map;function U1(n,e){let t=Be[e];if(t===void 0){const i=D1.get(e);if(i!==void 0)t=Be[i],console.warn('THREE.WebGLRenderer: Shader chunk "%s" has been deprecated. Use "%s" instead.',e,i);else throw new Error("Can not resolve #include <"+e+">")}return Uf(t)}const F1=/#pragma unroll_loop_start\s+for\s*\(\s*int\s+i\s*=\s*(\d+)\s*;\s*i\s*<\s*(\d+)\s*;\s*i\s*\+\+\s*\)\s*{([\s\S]+?)}\s+#pragma unroll_loop_end/g;function gg(n){return n.replace(F1,O1)}function O1(n,e,t,i){let r="";for(let s=parseInt(e);s<parseInt(t);s++)r+=i.replace(/\[\s*i\s*\]/g,"[ "+s+" ]").replace(/UNROLLED_LOOP_INDEX/g,s);return r}function _g(n){let e=`precision ${n.precision} float;
	precision ${n.precision} int;
	precision ${n.precision} sampler2D;
	precision ${n.precision} samplerCube;
	precision ${n.precision} sampler3D;
	precision ${n.precision} sampler2DArray;
	precision ${n.precision} sampler2DShadow;
	precision ${n.precision} samplerCubeShadow;
	precision ${n.precision} sampler2DArrayShadow;
	precision ${n.precision} isampler2D;
	precision ${n.precision} isampler3D;
	precision ${n.precision} isamplerCube;
	precision ${n.precision} isampler2DArray;
	precision ${n.precision} usampler2D;
	precision ${n.precision} usampler3D;
	precision ${n.precision} usamplerCube;
	precision ${n.precision} usampler2DArray;
	`;return n.precision==="highp"?e+=`
#define HIGH_PRECISION`:n.precision==="mediump"?e+=`
#define MEDIUM_PRECISION`:n.precision==="lowp"&&(e+=`
#define LOW_PRECISION`),e}function k1(n){let e="SHADOWMAP_TYPE_BASIC";return n.shadowMapType===_v?e="SHADOWMAP_TYPE_PCF":n.shadowMapType===eM?e="SHADOWMAP_TYPE_PCF_SOFT":n.shadowMapType===vi&&(e="SHADOWMAP_TYPE_VSM"),e}function B1(n){let e="ENVMAP_TYPE_CUBE";if(n.envMap)switch(n.envMapMode){case js:case Ys:e="ENVMAP_TYPE_CUBE";break;case zc:e="ENVMAP_TYPE_CUBE_UV";break}return e}function z1(n){let e="ENVMAP_MODE_REFLECTION";if(n.envMap)switch(n.envMapMode){case Ys:e="ENVMAP_MODE_REFRACTION";break}return e}function H1(n){let e="ENVMAP_BLENDING_NONE";if(n.envMap)switch(n.combine){case vv:e="ENVMAP_BLENDING_MULTIPLY";break;case AM:e="ENVMAP_BLENDING_MIX";break;case RM:e="ENVMAP_BLENDING_ADD";break}return e}function V1(n){const e=n.envMapCubeUVHeight;if(e===null)return null;const t=Math.log2(e)-2,i=1/e;return{texelWidth:1/(3*Math.max(Math.pow(2,t),7*16)),texelHeight:i,maxMip:t}}function G1(n,e,t,i){const r=n.getContext(),s=t.defines;let o=t.vertexShader,a=t.fragmentShader;const l=k1(t),c=B1(t),u=z1(t),d=H1(t),f=V1(t),p=P1(t),g=L1(s),y=r.createProgram();let m,h,_=t.glslVersion?"#version "+t.glslVersion+`
`:"";t.isRawShaderMaterial?(m=["#define SHADER_TYPE "+t.shaderType,"#define SHADER_NAME "+t.shaderName,g].filter(Oo).join(`
`),m.length>0&&(m+=`
`),h=["#define SHADER_TYPE "+t.shaderType,"#define SHADER_NAME "+t.shaderName,g].filter(Oo).join(`
`),h.length>0&&(h+=`
`)):(m=[_g(t),"#define SHADER_TYPE "+t.shaderType,"#define SHADER_NAME "+t.shaderName,g,t.extensionClipCullDistance?"#define USE_CLIP_DISTANCE":"",t.batching?"#define USE_BATCHING":"",t.batchingColor?"#define USE_BATCHING_COLOR":"",t.instancing?"#define USE_INSTANCING":"",t.instancingColor?"#define USE_INSTANCING_COLOR":"",t.instancingMorph?"#define USE_INSTANCING_MORPH":"",t.useFog&&t.fog?"#define USE_FOG":"",t.useFog&&t.fogExp2?"#define FOG_EXP2":"",t.map?"#define USE_MAP":"",t.envMap?"#define USE_ENVMAP":"",t.envMap?"#define "+u:"",t.lightMap?"#define USE_LIGHTMAP":"",t.aoMap?"#define USE_AOMAP":"",t.bumpMap?"#define USE_BUMPMAP":"",t.normalMap?"#define USE_NORMALMAP":"",t.normalMapObjectSpace?"#define USE_NORMALMAP_OBJECTSPACE":"",t.normalMapTangentSpace?"#define USE_NORMALMAP_TANGENTSPACE":"",t.displacementMap?"#define USE_DISPLACEMENTMAP":"",t.emissiveMap?"#define USE_EMISSIVEMAP":"",t.anisotropy?"#define USE_ANISOTROPY":"",t.anisotropyMap?"#define USE_ANISOTROPYMAP":"",t.clearcoatMap?"#define USE_CLEARCOATMAP":"",t.clearcoatRoughnessMap?"#define USE_CLEARCOAT_ROUGHNESSMAP":"",t.clearcoatNormalMap?"#define USE_CLEARCOAT_NORMALMAP":"",t.iridescenceMap?"#define USE_IRIDESCENCEMAP":"",t.iridescenceThicknessMap?"#define USE_IRIDESCENCE_THICKNESSMAP":"",t.specularMap?"#define USE_SPECULARMAP":"",t.specularColorMap?"#define USE_SPECULAR_COLORMAP":"",t.specularIntensityMap?"#define USE_SPECULAR_INTENSITYMAP":"",t.roughnessMap?"#define USE_ROUGHNESSMAP":"",t.metalnessMap?"#define USE_METALNESSMAP":"",t.alphaMap?"#define USE_ALPHAMAP":"",t.alphaHash?"#define USE_ALPHAHASH":"",t.transmission?"#define USE_TRANSMISSION":"",t.transmissionMap?"#define USE_TRANSMISSIONMAP":"",t.thicknessMap?"#define USE_THICKNESSMAP":"",t.sheenColorMap?"#define USE_SHEEN_COLORMAP":"",t.sheenRoughnessMap?"#define USE_SHEEN_ROUGHNESSMAP":"",t.mapUv?"#define MAP_UV "+t.mapUv:"",t.alphaMapUv?"#define ALPHAMAP_UV "+t.alphaMapUv:"",t.lightMapUv?"#define LIGHTMAP_UV "+t.lightMapUv:"",t.aoMapUv?"#define AOMAP_UV "+t.aoMapUv:"",t.emissiveMapUv?"#define EMISSIVEMAP_UV "+t.emissiveMapUv:"",t.bumpMapUv?"#define BUMPMAP_UV "+t.bumpMapUv:"",t.normalMapUv?"#define NORMALMAP_UV "+t.normalMapUv:"",t.displacementMapUv?"#define DISPLACEMENTMAP_UV "+t.displacementMapUv:"",t.metalnessMapUv?"#define METALNESSMAP_UV "+t.metalnessMapUv:"",t.roughnessMapUv?"#define ROUGHNESSMAP_UV "+t.roughnessMapUv:"",t.anisotropyMapUv?"#define ANISOTROPYMAP_UV "+t.anisotropyMapUv:"",t.clearcoatMapUv?"#define CLEARCOATMAP_UV "+t.clearcoatMapUv:"",t.clearcoatNormalMapUv?"#define CLEARCOAT_NORMALMAP_UV "+t.clearcoatNormalMapUv:"",t.clearcoatRoughnessMapUv?"#define CLEARCOAT_ROUGHNESSMAP_UV "+t.clearcoatRoughnessMapUv:"",t.iridescenceMapUv?"#define IRIDESCENCEMAP_UV "+t.iridescenceMapUv:"",t.iridescenceThicknessMapUv?"#define IRIDESCENCE_THICKNESSMAP_UV "+t.iridescenceThicknessMapUv:"",t.sheenColorMapUv?"#define SHEEN_COLORMAP_UV "+t.sheenColorMapUv:"",t.sheenRoughnessMapUv?"#define SHEEN_ROUGHNESSMAP_UV "+t.sheenRoughnessMapUv:"",t.specularMapUv?"#define SPECULARMAP_UV "+t.specularMapUv:"",t.specularColorMapUv?"#define SPECULAR_COLORMAP_UV "+t.specularColorMapUv:"",t.specularIntensityMapUv?"#define SPECULAR_INTENSITYMAP_UV "+t.specularIntensityMapUv:"",t.transmissionMapUv?"#define TRANSMISSIONMAP_UV "+t.transmissionMapUv:"",t.thicknessMapUv?"#define THICKNESSMAP_UV "+t.thicknessMapUv:"",t.vertexTangents&&t.flatShading===!1?"#define USE_TANGENT":"",t.vertexColors?"#define USE_COLOR":"",t.vertexAlphas?"#define USE_COLOR_ALPHA":"",t.vertexUv1s?"#define USE_UV1":"",t.vertexUv2s?"#define USE_UV2":"",t.vertexUv3s?"#define USE_UV3":"",t.pointsUvs?"#define USE_POINTS_UV":"",t.flatShading?"#define FLAT_SHADED":"",t.skinning?"#define USE_SKINNING":"",t.morphTargets?"#define USE_MORPHTARGETS":"",t.morphNormals&&t.flatShading===!1?"#define USE_MORPHNORMALS":"",t.morphColors?"#define USE_MORPHCOLORS":"",t.morphTargetsCount>0?"#define MORPHTARGETS_TEXTURE_STRIDE "+t.morphTextureStride:"",t.morphTargetsCount>0?"#define MORPHTARGETS_COUNT "+t.morphTargetsCount:"",t.doubleSided?"#define DOUBLE_SIDED":"",t.flipSided?"#define FLIP_SIDED":"",t.shadowMapEnabled?"#define USE_SHADOWMAP":"",t.shadowMapEnabled?"#define "+l:"",t.sizeAttenuation?"#define USE_SIZEATTENUATION":"",t.numLightProbes>0?"#define USE_LIGHT_PROBES":"",t.logarithmicDepthBuffer?"#define USE_LOGDEPTHBUF":"","uniform mat4 modelMatrix;","uniform mat4 modelViewMatrix;","uniform mat4 projectionMatrix;","uniform mat4 viewMatrix;","uniform mat3 normalMatrix;","uniform vec3 cameraPosition;","uniform bool isOrthographic;","#ifdef USE_INSTANCING","	attribute mat4 instanceMatrix;","#endif","#ifdef USE_INSTANCING_COLOR","	attribute vec3 instanceColor;","#endif","#ifdef USE_INSTANCING_MORPH","	uniform sampler2D morphTexture;","#endif","attribute vec3 position;","attribute vec3 normal;","attribute vec2 uv;","#ifdef USE_UV1","	attribute vec2 uv1;","#endif","#ifdef USE_UV2","	attribute vec2 uv2;","#endif","#ifdef USE_UV3","	attribute vec2 uv3;","#endif","#ifdef USE_TANGENT","	attribute vec4 tangent;","#endif","#if defined( USE_COLOR_ALPHA )","	attribute vec4 color;","#elif defined( USE_COLOR )","	attribute vec3 color;","#endif","#ifdef USE_SKINNING","	attribute vec4 skinIndex;","	attribute vec4 skinWeight;","#endif",`
`].filter(Oo).join(`
`),h=[_g(t),"#define SHADER_TYPE "+t.shaderType,"#define SHADER_NAME "+t.shaderName,g,t.useFog&&t.fog?"#define USE_FOG":"",t.useFog&&t.fogExp2?"#define FOG_EXP2":"",t.alphaToCoverage?"#define ALPHA_TO_COVERAGE":"",t.map?"#define USE_MAP":"",t.matcap?"#define USE_MATCAP":"",t.envMap?"#define USE_ENVMAP":"",t.envMap?"#define "+c:"",t.envMap?"#define "+u:"",t.envMap?"#define "+d:"",f?"#define CUBEUV_TEXEL_WIDTH "+f.texelWidth:"",f?"#define CUBEUV_TEXEL_HEIGHT "+f.texelHeight:"",f?"#define CUBEUV_MAX_MIP "+f.maxMip+".0":"",t.lightMap?"#define USE_LIGHTMAP":"",t.aoMap?"#define USE_AOMAP":"",t.bumpMap?"#define USE_BUMPMAP":"",t.normalMap?"#define USE_NORMALMAP":"",t.normalMapObjectSpace?"#define USE_NORMALMAP_OBJECTSPACE":"",t.normalMapTangentSpace?"#define USE_NORMALMAP_TANGENTSPACE":"",t.emissiveMap?"#define USE_EMISSIVEMAP":"",t.anisotropy?"#define USE_ANISOTROPY":"",t.anisotropyMap?"#define USE_ANISOTROPYMAP":"",t.clearcoat?"#define USE_CLEARCOAT":"",t.clearcoatMap?"#define USE_CLEARCOATMAP":"",t.clearcoatRoughnessMap?"#define USE_CLEARCOAT_ROUGHNESSMAP":"",t.clearcoatNormalMap?"#define USE_CLEARCOAT_NORMALMAP":"",t.dispersion?"#define USE_DISPERSION":"",t.iridescence?"#define USE_IRIDESCENCE":"",t.iridescenceMap?"#define USE_IRIDESCENCEMAP":"",t.iridescenceThicknessMap?"#define USE_IRIDESCENCE_THICKNESSMAP":"",t.specularMap?"#define USE_SPECULARMAP":"",t.specularColorMap?"#define USE_SPECULAR_COLORMAP":"",t.specularIntensityMap?"#define USE_SPECULAR_INTENSITYMAP":"",t.roughnessMap?"#define USE_ROUGHNESSMAP":"",t.metalnessMap?"#define USE_METALNESSMAP":"",t.alphaMap?"#define USE_ALPHAMAP":"",t.alphaTest?"#define USE_ALPHATEST":"",t.alphaHash?"#define USE_ALPHAHASH":"",t.sheen?"#define USE_SHEEN":"",t.sheenColorMap?"#define USE_SHEEN_COLORMAP":"",t.sheenRoughnessMap?"#define USE_SHEEN_ROUGHNESSMAP":"",t.transmission?"#define USE_TRANSMISSION":"",t.transmissionMap?"#define USE_TRANSMISSIONMAP":"",t.thicknessMap?"#define USE_THICKNESSMAP":"",t.vertexTangents&&t.flatShading===!1?"#define USE_TANGENT":"",t.vertexColors||t.instancingColor||t.batchingColor?"#define USE_COLOR":"",t.vertexAlphas?"#define USE_COLOR_ALPHA":"",t.vertexUv1s?"#define USE_UV1":"",t.vertexUv2s?"#define USE_UV2":"",t.vertexUv3s?"#define USE_UV3":"",t.pointsUvs?"#define USE_POINTS_UV":"",t.gradientMap?"#define USE_GRADIENTMAP":"",t.flatShading?"#define FLAT_SHADED":"",t.doubleSided?"#define DOUBLE_SIDED":"",t.flipSided?"#define FLIP_SIDED":"",t.shadowMapEnabled?"#define USE_SHADOWMAP":"",t.shadowMapEnabled?"#define "+l:"",t.premultipliedAlpha?"#define PREMULTIPLIED_ALPHA":"",t.numLightProbes>0?"#define USE_LIGHT_PROBES":"",t.decodeVideoTexture?"#define DECODE_VIDEO_TEXTURE":"",t.logarithmicDepthBuffer?"#define USE_LOGDEPTHBUF":"","uniform mat4 viewMatrix;","uniform vec3 cameraPosition;","uniform bool isOrthographic;",t.toneMapping!==ur?"#define TONE_MAPPING":"",t.toneMapping!==ur?Be.tonemapping_pars_fragment:"",t.toneMapping!==ur?C1("toneMapping",t.toneMapping):"",t.dithering?"#define DITHERING":"",t.opaque?"#define OPAQUE":"",Be.colorspace_pars_fragment,R1("linearToOutputTexel",t.outputColorSpace),b1(),t.useDepthPacking?"#define DEPTH_PACKING "+t.depthPacking:"",`
`].filter(Oo).join(`
`)),o=Uf(o),o=pg(o,t),o=mg(o,t),a=Uf(a),a=pg(a,t),a=mg(a,t),o=gg(o),a=gg(a),t.isRawShaderMaterial!==!0&&(_=`#version 300 es
`,m=[p,"#define attribute in","#define varying out","#define texture2D texture"].join(`
`)+`
`+m,h=["#define varying in",t.glslVersion===Lm?"":"layout(location = 0) out highp vec4 pc_fragColor;",t.glslVersion===Lm?"":"#define gl_FragColor pc_fragColor","#define gl_FragDepthEXT gl_FragDepth","#define texture2D texture","#define textureCube texture","#define texture2DProj textureProj","#define texture2DLodEXT textureLod","#define texture2DProjLodEXT textureProjLod","#define textureCubeLodEXT textureLod","#define texture2DGradEXT textureGrad","#define texture2DProjGradEXT textureProjGrad","#define textureCubeGradEXT textureGrad"].join(`
`)+`
`+h);const v=_+m+o,x=_+h+a,R=fg(r,r.VERTEX_SHADER,v),A=fg(r,r.FRAGMENT_SHADER,x);r.attachShader(y,R),r.attachShader(y,A),t.index0AttributeName!==void 0?r.bindAttribLocation(y,0,t.index0AttributeName):t.morphTargets===!0&&r.bindAttribLocation(y,0,"position"),r.linkProgram(y);function w(I){if(n.debug.checkShaderErrors){const V=r.getProgramInfoLog(y).trim(),B=r.getShaderInfoLog(R).trim(),j=r.getShaderInfoLog(A).trim();let Y=!0,W=!0;if(r.getProgramParameter(y,r.LINK_STATUS)===!1)if(Y=!1,typeof n.debug.onShaderError=="function")n.debug.onShaderError(r,y,R,A);else{const $=hg(r,R,"vertex"),N=hg(r,A,"fragment");console.error("THREE.WebGLProgram: Shader Error "+r.getError()+" - VALIDATE_STATUS "+r.getProgramParameter(y,r.VALIDATE_STATUS)+`

Material Name: `+I.name+`
Material Type: `+I.type+`

Program Info Log: `+V+`
`+$+`
`+N)}else V!==""?console.warn("THREE.WebGLProgram: Program Info Log:",V):(B===""||j==="")&&(W=!1);W&&(I.diagnostics={runnable:Y,programLog:V,vertexShader:{log:B,prefix:m},fragmentShader:{log:j,prefix:h}})}r.deleteShader(R),r.deleteShader(A),b=new Wl(r,y),T=I1(r,y)}let b;this.getUniforms=function(){return b===void 0&&w(this),b};let T;this.getAttributes=function(){return T===void 0&&w(this),T};let M=t.rendererExtensionParallelShaderCompile===!1;return this.isReady=function(){return M===!1&&(M=r.getProgramParameter(y,E1)),M},this.destroy=function(){i.releaseStatesOfProgram(this),r.deleteProgram(y),this.program=void 0},this.type=t.shaderType,this.name=t.shaderName,this.id=T1++,this.cacheKey=e,this.usedTimes=1,this.program=y,this.vertexShader=R,this.fragmentShader=A,this}let W1=0;class X1{constructor(){this.shaderCache=new Map,this.materialCache=new Map}update(e){const t=e.vertexShader,i=e.fragmentShader,r=this._getShaderStage(t),s=this._getShaderStage(i),o=this._getShaderCacheForMaterial(e);return o.has(r)===!1&&(o.add(r),r.usedTimes++),o.has(s)===!1&&(o.add(s),s.usedTimes++),this}remove(e){const t=this.materialCache.get(e);for(const i of t)i.usedTimes--,i.usedTimes===0&&this.shaderCache.delete(i.code);return this.materialCache.delete(e),this}getVertexShaderID(e){return this._getShaderStage(e.vertexShader).id}getFragmentShaderID(e){return this._getShaderStage(e.fragmentShader).id}dispose(){this.shaderCache.clear(),this.materialCache.clear()}_getShaderCacheForMaterial(e){const t=this.materialCache;let i=t.get(e);return i===void 0&&(i=new Set,t.set(e,i)),i}_getShaderStage(e){const t=this.shaderCache;let i=t.get(e);return i===void 0&&(i=new j1(e),t.set(e,i)),i}}class j1{constructor(e){this.id=W1++,this.code=e,this.usedTimes=0}}function Y1(n,e,t,i,r,s,o){const a=new Yh,l=new X1,c=new Set,u=[],d=r.logarithmicDepthBuffer,f=r.vertexTextures;let p=r.precision;const g={MeshDepthMaterial:"depth",MeshDistanceMaterial:"distanceRGBA",MeshNormalMaterial:"normal",MeshBasicMaterial:"basic",MeshLambertMaterial:"lambert",MeshPhongMaterial:"phong",MeshToonMaterial:"toon",MeshStandardMaterial:"physical",MeshPhysicalMaterial:"physical",MeshMatcapMaterial:"matcap",LineBasicMaterial:"basic",LineDashedMaterial:"dashed",PointsMaterial:"points",ShadowMaterial:"shadow",SpriteMaterial:"sprite"};function y(T){return c.add(T),T===0?"uv":`uv${T}`}function m(T,M,I,V,B){const j=V.fog,Y=B.geometry,W=T.isMeshStandardMaterial?V.environment:null,$=(T.isMeshStandardMaterial?t:e).get(T.envMap||W),N=$&&$.mapping===zc?$.image.height:null,K=g[T.type];T.precision!==null&&(p=r.getMaxPrecision(T.precision),p!==T.precision&&console.warn("THREE.WebGLProgram.getParameters:",T.precision,"not supported, using",p,"instead."));const Q=Y.morphAttributes.position||Y.morphAttributes.normal||Y.morphAttributes.color,oe=Q!==void 0?Q.length:0;let Me=0;Y.morphAttributes.position!==void 0&&(Me=1),Y.morphAttributes.normal!==void 0&&(Me=2),Y.morphAttributes.color!==void 0&&(Me=3);let je,G,ne,he;if(K){const $e=ei[K];je=$e.vertexShader,G=$e.fragmentShader}else je=T.vertexShader,G=T.fragmentShader,l.update(T),ne=l.getVertexShaderID(T),he=l.getFragmentShaderID(T);const de=n.getRenderTarget(),Ce=B.isInstancedMesh===!0,Fe=B.isBatchedMesh===!0,We=!!T.map,gt=!!T.matcap,P=!!$,St=!!T.aoMap,et=!!T.lightMap,rt=!!T.bumpMap,Ee=!!T.normalMap,Mt=!!T.displacementMap,Ie=!!T.emissiveMap,Oe=!!T.metalnessMap,C=!!T.roughnessMap,S=T.anisotropy>0,z=T.clearcoat>0,Z=T.dispersion>0,ee=T.iridescence>0,J=T.sheen>0,Te=T.transmission>0,ce=S&&!!T.anisotropyMap,ge=z&&!!T.clearcoatMap,ke=z&&!!T.clearcoatNormalMap,ie=z&&!!T.clearcoatRoughnessMap,me=ee&&!!T.iridescenceMap,Xe=ee&&!!T.iridescenceThicknessMap,Le=J&&!!T.sheenColorMap,_e=J&&!!T.sheenRoughnessMap,De=!!T.specularMap,He=!!T.specularColorMap,ft=!!T.specularIntensityMap,D=Te&&!!T.transmissionMap,re=Te&&!!T.thicknessMap,X=!!T.gradientMap,q=!!T.alphaMap,ae=T.alphaTest>0,Ae=!!T.alphaHash,Ye=!!T.extensions;let Et=ur;T.toneMapped&&(de===null||de.isXRRenderTarget===!0)&&(Et=n.toneMapping);const Ot={shaderID:K,shaderType:T.type,shaderName:T.name,vertexShader:je,fragmentShader:G,defines:T.defines,customVertexShaderID:ne,customFragmentShaderID:he,isRawShaderMaterial:T.isRawShaderMaterial===!0,glslVersion:T.glslVersion,precision:p,batching:Fe,batchingColor:Fe&&B._colorsTexture!==null,instancing:Ce,instancingColor:Ce&&B.instanceColor!==null,instancingMorph:Ce&&B.morphTexture!==null,supportsVertexTextures:f,outputColorSpace:de===null?n.outputColorSpace:de.isXRRenderTarget===!0?de.texture.colorSpace:Vt,alphaToCoverage:!!T.alphaToCoverage,map:We,matcap:gt,envMap:P,envMapMode:P&&$.mapping,envMapCubeUVHeight:N,aoMap:St,lightMap:et,bumpMap:rt,normalMap:Ee,displacementMap:f&&Mt,emissiveMap:Ie,normalMapObjectSpace:Ee&&T.normalMapType===zM,normalMapTangentSpace:Ee&&T.normalMapType===Lv,metalnessMap:Oe,roughnessMap:C,anisotropy:S,anisotropyMap:ce,clearcoat:z,clearcoatMap:ge,clearcoatNormalMap:ke,clearcoatRoughnessMap:ie,dispersion:Z,iridescence:ee,iridescenceMap:me,iridescenceThicknessMap:Xe,sheen:J,sheenColorMap:Le,sheenRoughnessMap:_e,specularMap:De,specularColorMap:He,specularIntensityMap:ft,transmission:Te,transmissionMap:D,thicknessMap:re,gradientMap:X,opaque:T.transparent===!1&&T.blending===Ds&&T.alphaToCoverage===!1,alphaMap:q,alphaTest:ae,alphaHash:Ae,combine:T.combine,mapUv:We&&y(T.map.channel),aoMapUv:St&&y(T.aoMap.channel),lightMapUv:et&&y(T.lightMap.channel),bumpMapUv:rt&&y(T.bumpMap.channel),normalMapUv:Ee&&y(T.normalMap.channel),displacementMapUv:Mt&&y(T.displacementMap.channel),emissiveMapUv:Ie&&y(T.emissiveMap.channel),metalnessMapUv:Oe&&y(T.metalnessMap.channel),roughnessMapUv:C&&y(T.roughnessMap.channel),anisotropyMapUv:ce&&y(T.anisotropyMap.channel),clearcoatMapUv:ge&&y(T.clearcoatMap.channel),clearcoatNormalMapUv:ke&&y(T.clearcoatNormalMap.channel),clearcoatRoughnessMapUv:ie&&y(T.clearcoatRoughnessMap.channel),iridescenceMapUv:me&&y(T.iridescenceMap.channel),iridescenceThicknessMapUv:Xe&&y(T.iridescenceThicknessMap.channel),sheenColorMapUv:Le&&y(T.sheenColorMap.channel),sheenRoughnessMapUv:_e&&y(T.sheenRoughnessMap.channel),specularMapUv:De&&y(T.specularMap.channel),specularColorMapUv:He&&y(T.specularColorMap.channel),specularIntensityMapUv:ft&&y(T.specularIntensityMap.channel),transmissionMapUv:D&&y(T.transmissionMap.channel),thicknessMapUv:re&&y(T.thicknessMap.channel),alphaMapUv:q&&y(T.alphaMap.channel),vertexTangents:!!Y.attributes.tangent&&(Ee||S),vertexColors:T.vertexColors,vertexAlphas:T.vertexColors===!0&&!!Y.attributes.color&&Y.attributes.color.itemSize===4,pointsUvs:B.isPoints===!0&&!!Y.attributes.uv&&(We||q),fog:!!j,useFog:T.fog===!0,fogExp2:!!j&&j.isFogExp2,flatShading:T.flatShading===!0,sizeAttenuation:T.sizeAttenuation===!0,logarithmicDepthBuffer:d,skinning:B.isSkinnedMesh===!0,morphTargets:Y.morphAttributes.position!==void 0,morphNormals:Y.morphAttributes.normal!==void 0,morphColors:Y.morphAttributes.color!==void 0,morphTargetsCount:oe,morphTextureStride:Me,numDirLights:M.directional.length,numPointLights:M.point.length,numSpotLights:M.spot.length,numSpotLightMaps:M.spotLightMap.length,numRectAreaLights:M.rectArea.length,numHemiLights:M.hemi.length,numDirLightShadows:M.directionalShadowMap.length,numPointLightShadows:M.pointShadowMap.length,numSpotLightShadows:M.spotShadowMap.length,numSpotLightShadowsWithMaps:M.numSpotLightShadowsWithMaps,numLightProbes:M.numLightProbes,numClippingPlanes:o.numPlanes,numClipIntersection:o.numIntersection,dithering:T.dithering,shadowMapEnabled:n.shadowMap.enabled&&I.length>0,shadowMapType:n.shadowMap.type,toneMapping:Et,decodeVideoTexture:We&&T.map.isVideoTexture===!0&&qe.getTransfer(T.map.colorSpace)===lt,premultipliedAlpha:T.premultipliedAlpha,doubleSided:T.side===ii,flipSided:T.side===fn,useDepthPacking:T.depthPacking>=0,depthPacking:T.depthPacking||0,index0AttributeName:T.index0AttributeName,extensionClipCullDistance:Ye&&T.extensions.clipCullDistance===!0&&i.has("WEBGL_clip_cull_distance"),extensionMultiDraw:(Ye&&T.extensions.multiDraw===!0||Fe)&&i.has("WEBGL_multi_draw"),rendererExtensionParallelShaderCompile:i.has("KHR_parallel_shader_compile"),customProgramCacheKey:T.customProgramCacheKey()};return Ot.vertexUv1s=c.has(1),Ot.vertexUv2s=c.has(2),Ot.vertexUv3s=c.has(3),c.clear(),Ot}function h(T){const M=[];if(T.shaderID?M.push(T.shaderID):(M.push(T.customVertexShaderID),M.push(T.customFragmentShaderID)),T.defines!==void 0)for(const I in T.defines)M.push(I),M.push(T.defines[I]);return T.isRawShaderMaterial===!1&&(_(M,T),v(M,T),M.push(n.outputColorSpace)),M.push(T.customProgramCacheKey),M.join()}function _(T,M){T.push(M.precision),T.push(M.outputColorSpace),T.push(M.envMapMode),T.push(M.envMapCubeUVHeight),T.push(M.mapUv),T.push(M.alphaMapUv),T.push(M.lightMapUv),T.push(M.aoMapUv),T.push(M.bumpMapUv),T.push(M.normalMapUv),T.push(M.displacementMapUv),T.push(M.emissiveMapUv),T.push(M.metalnessMapUv),T.push(M.roughnessMapUv),T.push(M.anisotropyMapUv),T.push(M.clearcoatMapUv),T.push(M.clearcoatNormalMapUv),T.push(M.clearcoatRoughnessMapUv),T.push(M.iridescenceMapUv),T.push(M.iridescenceThicknessMapUv),T.push(M.sheenColorMapUv),T.push(M.sheenRoughnessMapUv),T.push(M.specularMapUv),T.push(M.specularColorMapUv),T.push(M.specularIntensityMapUv),T.push(M.transmissionMapUv),T.push(M.thicknessMapUv),T.push(M.combine),T.push(M.fogExp2),T.push(M.sizeAttenuation),T.push(M.morphTargetsCount),T.push(M.morphAttributeCount),T.push(M.numDirLights),T.push(M.numPointLights),T.push(M.numSpotLights),T.push(M.numSpotLightMaps),T.push(M.numHemiLights),T.push(M.numRectAreaLights),T.push(M.numDirLightShadows),T.push(M.numPointLightShadows),T.push(M.numSpotLightShadows),T.push(M.numSpotLightShadowsWithMaps),T.push(M.numLightProbes),T.push(M.shadowMapType),T.push(M.toneMapping),T.push(M.numClippingPlanes),T.push(M.numClipIntersection),T.push(M.depthPacking)}function v(T,M){a.disableAll(),M.supportsVertexTextures&&a.enable(0),M.instancing&&a.enable(1),M.instancingColor&&a.enable(2),M.instancingMorph&&a.enable(3),M.matcap&&a.enable(4),M.envMap&&a.enable(5),M.normalMapObjectSpace&&a.enable(6),M.normalMapTangentSpace&&a.enable(7),M.clearcoat&&a.enable(8),M.iridescence&&a.enable(9),M.alphaTest&&a.enable(10),M.vertexColors&&a.enable(11),M.vertexAlphas&&a.enable(12),M.vertexUv1s&&a.enable(13),M.vertexUv2s&&a.enable(14),M.vertexUv3s&&a.enable(15),M.vertexTangents&&a.enable(16),M.anisotropy&&a.enable(17),M.alphaHash&&a.enable(18),M.batching&&a.enable(19),M.dispersion&&a.enable(20),M.batchingColor&&a.enable(21),T.push(a.mask),a.disableAll(),M.fog&&a.enable(0),M.useFog&&a.enable(1),M.flatShading&&a.enable(2),M.logarithmicDepthBuffer&&a.enable(3),M.skinning&&a.enable(4),M.morphTargets&&a.enable(5),M.morphNormals&&a.enable(6),M.morphColors&&a.enable(7),M.premultipliedAlpha&&a.enable(8),M.shadowMapEnabled&&a.enable(9),M.doubleSided&&a.enable(10),M.flipSided&&a.enable(11),M.useDepthPacking&&a.enable(12),M.dithering&&a.enable(13),M.transmission&&a.enable(14),M.sheen&&a.enable(15),M.opaque&&a.enable(16),M.pointsUvs&&a.enable(17),M.decodeVideoTexture&&a.enable(18),M.alphaToCoverage&&a.enable(19),T.push(a.mask)}function x(T){const M=g[T.type];let I;if(M){const V=ei[M];I=PE.clone(V.uniforms)}else I=T.uniforms;return I}function R(T,M){let I;for(let V=0,B=u.length;V<B;V++){const j=u[V];if(j.cacheKey===M){I=j,++I.usedTimes;break}}return I===void 0&&(I=new G1(n,M,T,s),u.push(I)),I}function A(T){if(--T.usedTimes===0){const M=u.indexOf(T);u[M]=u[u.length-1],u.pop(),T.destroy()}}function w(T){l.remove(T)}function b(){l.dispose()}return{getParameters:m,getProgramCacheKey:h,getUniforms:x,acquireProgram:R,releaseProgram:A,releaseShaderCache:w,programs:u,dispose:b}}function K1(){let n=new WeakMap;function e(o){return n.has(o)}function t(o){let a=n.get(o);return a===void 0&&(a={},n.set(o,a)),a}function i(o){n.delete(o)}function r(o,a,l){n.get(o)[a]=l}function s(){n=new WeakMap}return{has:e,get:t,remove:i,update:r,dispose:s}}function q1(n,e){return n.groupOrder!==e.groupOrder?n.groupOrder-e.groupOrder:n.renderOrder!==e.renderOrder?n.renderOrder-e.renderOrder:n.material.id!==e.material.id?n.material.id-e.material.id:n.z!==e.z?n.z-e.z:n.id-e.id}function vg(n,e){return n.groupOrder!==e.groupOrder?n.groupOrder-e.groupOrder:n.renderOrder!==e.renderOrder?n.renderOrder-e.renderOrder:n.z!==e.z?e.z-n.z:n.id-e.id}function yg(){const n=[];let e=0;const t=[],i=[],r=[];function s(){e=0,t.length=0,i.length=0,r.length=0}function o(d,f,p,g,y,m){let h=n[e];return h===void 0?(h={id:d.id,object:d,geometry:f,material:p,groupOrder:g,renderOrder:d.renderOrder,z:y,group:m},n[e]=h):(h.id=d.id,h.object=d,h.geometry=f,h.material=p,h.groupOrder=g,h.renderOrder=d.renderOrder,h.z=y,h.group=m),e++,h}function a(d,f,p,g,y,m){const h=o(d,f,p,g,y,m);p.transmission>0?i.push(h):p.transparent===!0?r.push(h):t.push(h)}function l(d,f,p,g,y,m){const h=o(d,f,p,g,y,m);p.transmission>0?i.unshift(h):p.transparent===!0?r.unshift(h):t.unshift(h)}function c(d,f){t.length>1&&t.sort(d||q1),i.length>1&&i.sort(f||vg),r.length>1&&r.sort(f||vg)}function u(){for(let d=e,f=n.length;d<f;d++){const p=n[d];if(p.id===null)break;p.id=null,p.object=null,p.geometry=null,p.material=null,p.group=null}}return{opaque:t,transmissive:i,transparent:r,init:s,push:a,unshift:l,finish:u,sort:c}}function $1(){let n=new WeakMap;function e(i,r){const s=n.get(i);let o;return s===void 0?(o=new yg,n.set(i,[o])):r>=s.length?(o=new yg,s.push(o)):o=s[r],o}function t(){n=new WeakMap}return{get:e,dispose:t}}function Z1(){const n={};return{get:function(e){if(n[e.id]!==void 0)return n[e.id];let t;switch(e.type){case"DirectionalLight":t={direction:new L,color:new Pe};break;case"SpotLight":t={position:new L,direction:new L,color:new Pe,distance:0,coneCos:0,penumbraCos:0,decay:0};break;case"PointLight":t={position:new L,color:new Pe,distance:0,decay:0};break;case"HemisphereLight":t={direction:new L,skyColor:new Pe,groundColor:new Pe};break;case"RectAreaLight":t={color:new Pe,position:new L,halfWidth:new L,halfHeight:new L};break}return n[e.id]=t,t}}}function Q1(){const n={};return{get:function(e){if(n[e.id]!==void 0)return n[e.id];let t;switch(e.type){case"DirectionalLight":t={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new we};break;case"SpotLight":t={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new we};break;case"PointLight":t={shadowIntensity:1,shadowBias:0,shadowNormalBias:0,shadowRadius:1,shadowMapSize:new we,shadowCameraNear:1,shadowCameraFar:1e3};break}return n[e.id]=t,t}}}let J1=0;function eR(n,e){return(e.castShadow?2:0)-(n.castShadow?2:0)+(e.map?1:0)-(n.map?1:0)}function tR(n){const e=new Z1,t=Q1(),i={version:0,hash:{directionalLength:-1,pointLength:-1,spotLength:-1,rectAreaLength:-1,hemiLength:-1,numDirectionalShadows:-1,numPointShadows:-1,numSpotShadows:-1,numSpotMaps:-1,numLightProbes:-1},ambient:[0,0,0],probe:[],directional:[],directionalShadow:[],directionalShadowMap:[],directionalShadowMatrix:[],spot:[],spotLightMap:[],spotShadow:[],spotShadowMap:[],spotLightMatrix:[],rectArea:[],rectAreaLTC1:null,rectAreaLTC2:null,point:[],pointShadow:[],pointShadowMap:[],pointShadowMatrix:[],hemi:[],numSpotLightShadowsWithMaps:0,numLightProbes:0};for(let c=0;c<9;c++)i.probe.push(new L);const r=new L,s=new Ue,o=new Ue;function a(c){let u=0,d=0,f=0;for(let T=0;T<9;T++)i.probe[T].set(0,0,0);let p=0,g=0,y=0,m=0,h=0,_=0,v=0,x=0,R=0,A=0,w=0;c.sort(eR);for(let T=0,M=c.length;T<M;T++){const I=c[T],V=I.color,B=I.intensity,j=I.distance,Y=I.shadow&&I.shadow.map?I.shadow.map.texture:null;if(I.isAmbientLight)u+=V.r*B,d+=V.g*B,f+=V.b*B;else if(I.isLightProbe){for(let W=0;W<9;W++)i.probe[W].addScaledVector(I.sh.coefficients[W],B);w++}else if(I.isDirectionalLight){const W=e.get(I);if(W.color.copy(I.color).multiplyScalar(I.intensity),I.castShadow){const $=I.shadow,N=t.get(I);N.shadowIntensity=$.intensity,N.shadowBias=$.bias,N.shadowNormalBias=$.normalBias,N.shadowRadius=$.radius,N.shadowMapSize=$.mapSize,i.directionalShadow[p]=N,i.directionalShadowMap[p]=Y,i.directionalShadowMatrix[p]=I.shadow.matrix,_++}i.directional[p]=W,p++}else if(I.isSpotLight){const W=e.get(I);W.position.setFromMatrixPosition(I.matrixWorld),W.color.copy(V).multiplyScalar(B),W.distance=j,W.coneCos=Math.cos(I.angle),W.penumbraCos=Math.cos(I.angle*(1-I.penumbra)),W.decay=I.decay,i.spot[y]=W;const $=I.shadow;if(I.map&&(i.spotLightMap[R]=I.map,R++,$.updateMatrices(I),I.castShadow&&A++),i.spotLightMatrix[y]=$.matrix,I.castShadow){const N=t.get(I);N.shadowIntensity=$.intensity,N.shadowBias=$.bias,N.shadowNormalBias=$.normalBias,N.shadowRadius=$.radius,N.shadowMapSize=$.mapSize,i.spotShadow[y]=N,i.spotShadowMap[y]=Y,x++}y++}else if(I.isRectAreaLight){const W=e.get(I);W.color.copy(V).multiplyScalar(B),W.halfWidth.set(I.width*.5,0,0),W.halfHeight.set(0,I.height*.5,0),i.rectArea[m]=W,m++}else if(I.isPointLight){const W=e.get(I);if(W.color.copy(I.color).multiplyScalar(I.intensity),W.distance=I.distance,W.decay=I.decay,I.castShadow){const $=I.shadow,N=t.get(I);N.shadowIntensity=$.intensity,N.shadowBias=$.bias,N.shadowNormalBias=$.normalBias,N.shadowRadius=$.radius,N.shadowMapSize=$.mapSize,N.shadowCameraNear=$.camera.near,N.shadowCameraFar=$.camera.far,i.pointShadow[g]=N,i.pointShadowMap[g]=Y,i.pointShadowMatrix[g]=I.shadow.matrix,v++}i.point[g]=W,g++}else if(I.isHemisphereLight){const W=e.get(I);W.skyColor.copy(I.color).multiplyScalar(B),W.groundColor.copy(I.groundColor).multiplyScalar(B),i.hemi[h]=W,h++}}m>0&&(n.has("OES_texture_float_linear")===!0?(i.rectAreaLTC1=le.LTC_FLOAT_1,i.rectAreaLTC2=le.LTC_FLOAT_2):(i.rectAreaLTC1=le.LTC_HALF_1,i.rectAreaLTC2=le.LTC_HALF_2)),i.ambient[0]=u,i.ambient[1]=d,i.ambient[2]=f;const b=i.hash;(b.directionalLength!==p||b.pointLength!==g||b.spotLength!==y||b.rectAreaLength!==m||b.hemiLength!==h||b.numDirectionalShadows!==_||b.numPointShadows!==v||b.numSpotShadows!==x||b.numSpotMaps!==R||b.numLightProbes!==w)&&(i.directional.length=p,i.spot.length=y,i.rectArea.length=m,i.point.length=g,i.hemi.length=h,i.directionalShadow.length=_,i.directionalShadowMap.length=_,i.pointShadow.length=v,i.pointShadowMap.length=v,i.spotShadow.length=x,i.spotShadowMap.length=x,i.directionalShadowMatrix.length=_,i.pointShadowMatrix.length=v,i.spotLightMatrix.length=x+R-A,i.spotLightMap.length=R,i.numSpotLightShadowsWithMaps=A,i.numLightProbes=w,b.directionalLength=p,b.pointLength=g,b.spotLength=y,b.rectAreaLength=m,b.hemiLength=h,b.numDirectionalShadows=_,b.numPointShadows=v,b.numSpotShadows=x,b.numSpotMaps=R,b.numLightProbes=w,i.version=J1++)}function l(c,u){let d=0,f=0,p=0,g=0,y=0;const m=u.matrixWorldInverse;for(let h=0,_=c.length;h<_;h++){const v=c[h];if(v.isDirectionalLight){const x=i.directional[d];x.direction.setFromMatrixPosition(v.matrixWorld),r.setFromMatrixPosition(v.target.matrixWorld),x.direction.sub(r),x.direction.transformDirection(m),d++}else if(v.isSpotLight){const x=i.spot[p];x.position.setFromMatrixPosition(v.matrixWorld),x.position.applyMatrix4(m),x.direction.setFromMatrixPosition(v.matrixWorld),r.setFromMatrixPosition(v.target.matrixWorld),x.direction.sub(r),x.direction.transformDirection(m),p++}else if(v.isRectAreaLight){const x=i.rectArea[g];x.position.setFromMatrixPosition(v.matrixWorld),x.position.applyMatrix4(m),o.identity(),s.copy(v.matrixWorld),s.premultiply(m),o.extractRotation(s),x.halfWidth.set(v.width*.5,0,0),x.halfHeight.set(0,v.height*.5,0),x.halfWidth.applyMatrix4(o),x.halfHeight.applyMatrix4(o),g++}else if(v.isPointLight){const x=i.point[f];x.position.setFromMatrixPosition(v.matrixWorld),x.position.applyMatrix4(m),f++}else if(v.isHemisphereLight){const x=i.hemi[y];x.direction.setFromMatrixPosition(v.matrixWorld),x.direction.transformDirection(m),y++}}}return{setup:a,setupView:l,state:i}}function xg(n){const e=new tR(n),t=[],i=[];function r(u){c.camera=u,t.length=0,i.length=0}function s(u){t.push(u)}function o(u){i.push(u)}function a(){e.setup(t)}function l(u){e.setupView(t,u)}const c={lightsArray:t,shadowsArray:i,camera:null,lights:e,transmissionRenderTarget:{}};return{init:r,state:c,setupLights:a,setupLightsView:l,pushLight:s,pushShadow:o}}function nR(n){let e=new WeakMap;function t(r,s=0){const o=e.get(r);let a;return o===void 0?(a=new xg(n),e.set(r,[a])):s>=o.length?(a=new xg(n),o.push(a)):a=o[s],a}function i(){e=new WeakMap}return{get:t,dispose:i}}class iR extends qn{constructor(e){super(),this.isMeshDepthMaterial=!0,this.type="MeshDepthMaterial",this.depthPacking=kM,this.map=null,this.alphaMap=null,this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.wireframe=!1,this.wireframeLinewidth=1,this.setValues(e)}copy(e){return super.copy(e),this.depthPacking=e.depthPacking,this.map=e.map,this.alphaMap=e.alphaMap,this.displacementMap=e.displacementMap,this.displacementScale=e.displacementScale,this.displacementBias=e.displacementBias,this.wireframe=e.wireframe,this.wireframeLinewidth=e.wireframeLinewidth,this}}class rR extends qn{constructor(e){super(),this.isMeshDistanceMaterial=!0,this.type="MeshDistanceMaterial",this.map=null,this.alphaMap=null,this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.setValues(e)}copy(e){return super.copy(e),this.map=e.map,this.alphaMap=e.alphaMap,this.displacementMap=e.displacementMap,this.displacementScale=e.displacementScale,this.displacementBias=e.displacementBias,this}}const sR=`void main() {
	gl_Position = vec4( position, 1.0 );
}`,oR=`uniform sampler2D shadow_pass;
uniform vec2 resolution;
uniform float radius;
#include <packing>
void main() {
	const float samples = float( VSM_SAMPLES );
	float mean = 0.0;
	float squared_mean = 0.0;
	float uvStride = samples <= 1.0 ? 0.0 : 2.0 / ( samples - 1.0 );
	float uvStart = samples <= 1.0 ? 0.0 : - 1.0;
	for ( float i = 0.0; i < samples; i ++ ) {
		float uvOffset = uvStart + i * uvStride;
		#ifdef HORIZONTAL_PASS
			vec2 distribution = unpackRGBATo2Half( texture2D( shadow_pass, ( gl_FragCoord.xy + vec2( uvOffset, 0.0 ) * radius ) / resolution ) );
			mean += distribution.x;
			squared_mean += distribution.y * distribution.y + distribution.x * distribution.x;
		#else
			float depth = unpackRGBAToDepth( texture2D( shadow_pass, ( gl_FragCoord.xy + vec2( 0.0, uvOffset ) * radius ) / resolution ) );
			mean += depth;
			squared_mean += depth * depth;
		#endif
	}
	mean = mean / samples;
	squared_mean = squared_mean / samples;
	float std_dev = sqrt( squared_mean - mean * mean );
	gl_FragColor = pack2HalfToRGBA( vec2( mean, std_dev ) );
}`;function aR(n,e,t){let i=new Kh;const r=new we,s=new we,o=new st,a=new iR({depthPacking:BM}),l=new rR,c={},u=t.maxTextureSize,d={[Ii]:fn,[fn]:Ii,[ii]:ii},f=new Di({defines:{VSM_SAMPLES:8},uniforms:{shadow_pass:{value:null},resolution:{value:new we},radius:{value:4}},vertexShader:sR,fragmentShader:oR}),p=f.clone();p.defines.HORIZONTAL_PASS=1;const g=new hn;g.setAttribute("position",new Rt(new Float32Array([-1,-1,.5,3,-1,.5,-1,3,.5]),3));const y=new tn(g,f),m=this;this.enabled=!1,this.autoUpdate=!0,this.needsUpdate=!1,this.type=_v;let h=this.type;this.render=function(A,w,b){if(m.enabled===!1||m.autoUpdate===!1&&m.needsUpdate===!1||A.length===0)return;const T=n.getRenderTarget(),M=n.getActiveCubeFace(),I=n.getActiveMipmapLevel(),V=n.state;V.setBlending(cr),V.buffers.color.setClear(1,1,1,1),V.buffers.depth.setTest(!0),V.setScissorTest(!1);const B=h!==vi&&this.type===vi,j=h===vi&&this.type!==vi;for(let Y=0,W=A.length;Y<W;Y++){const $=A[Y],N=$.shadow;if(N===void 0){console.warn("THREE.WebGLShadowMap:",$,"has no shadow.");continue}if(N.autoUpdate===!1&&N.needsUpdate===!1)continue;r.copy(N.mapSize);const K=N.getFrameExtents();if(r.multiply(K),s.copy(N.mapSize),(r.x>u||r.y>u)&&(r.x>u&&(s.x=Math.floor(u/K.x),r.x=s.x*K.x,N.mapSize.x=s.x),r.y>u&&(s.y=Math.floor(u/K.y),r.y=s.y*K.y,N.mapSize.y=s.y)),N.map===null||B===!0||j===!0){const oe=this.type!==vi?{minFilter:en,magFilter:en}:{};N.map!==null&&N.map.dispose(),N.map=new Vr(r.x,r.y,oe),N.map.texture.name=$.name+".shadowMap",N.camera.updateProjectionMatrix()}n.setRenderTarget(N.map),n.clear();const Q=N.getViewportCount();for(let oe=0;oe<Q;oe++){const Me=N.getViewport(oe);o.set(s.x*Me.x,s.y*Me.y,s.x*Me.z,s.y*Me.w),V.viewport(o),N.updateMatrices($,oe),i=N.getFrustum(),x(w,b,N.camera,$,this.type)}N.isPointLightShadow!==!0&&this.type===vi&&_(N,b),N.needsUpdate=!1}h=this.type,m.needsUpdate=!1,n.setRenderTarget(T,M,I)};function _(A,w){const b=e.update(y);f.defines.VSM_SAMPLES!==A.blurSamples&&(f.defines.VSM_SAMPLES=A.blurSamples,p.defines.VSM_SAMPLES=A.blurSamples,f.needsUpdate=!0,p.needsUpdate=!0),A.mapPass===null&&(A.mapPass=new Vr(r.x,r.y)),f.uniforms.shadow_pass.value=A.map.texture,f.uniforms.resolution.value=A.mapSize,f.uniforms.radius.value=A.radius,n.setRenderTarget(A.mapPass),n.clear(),n.renderBufferDirect(w,null,b,f,y,null),p.uniforms.shadow_pass.value=A.mapPass.texture,p.uniforms.resolution.value=A.mapSize,p.uniforms.radius.value=A.radius,n.setRenderTarget(A.map),n.clear(),n.renderBufferDirect(w,null,b,p,y,null)}function v(A,w,b,T){let M=null;const I=b.isPointLight===!0?A.customDistanceMaterial:A.customDepthMaterial;if(I!==void 0)M=I;else if(M=b.isPointLight===!0?l:a,n.localClippingEnabled&&w.clipShadows===!0&&Array.isArray(w.clippingPlanes)&&w.clippingPlanes.length!==0||w.displacementMap&&w.displacementScale!==0||w.alphaMap&&w.alphaTest>0||w.map&&w.alphaTest>0){const V=M.uuid,B=w.uuid;let j=c[V];j===void 0&&(j={},c[V]=j);let Y=j[B];Y===void 0&&(Y=M.clone(),j[B]=Y,w.addEventListener("dispose",R)),M=Y}if(M.visible=w.visible,M.wireframe=w.wireframe,T===vi?M.side=w.shadowSide!==null?w.shadowSide:w.side:M.side=w.shadowSide!==null?w.shadowSide:d[w.side],M.alphaMap=w.alphaMap,M.alphaTest=w.alphaTest,M.map=w.map,M.clipShadows=w.clipShadows,M.clippingPlanes=w.clippingPlanes,M.clipIntersection=w.clipIntersection,M.displacementMap=w.displacementMap,M.displacementScale=w.displacementScale,M.displacementBias=w.displacementBias,M.wireframeLinewidth=w.wireframeLinewidth,M.linewidth=w.linewidth,b.isPointLight===!0&&M.isMeshDistanceMaterial===!0){const V=n.properties.get(M);V.light=b}return M}function x(A,w,b,T,M){if(A.visible===!1)return;if(A.layers.test(w.layers)&&(A.isMesh||A.isLine||A.isPoints)&&(A.castShadow||A.receiveShadow&&M===vi)&&(!A.frustumCulled||i.intersectsObject(A))){A.modelViewMatrix.multiplyMatrices(b.matrixWorldInverse,A.matrixWorld);const B=e.update(A),j=A.material;if(Array.isArray(j)){const Y=B.groups;for(let W=0,$=Y.length;W<$;W++){const N=Y[W],K=j[N.materialIndex];if(K&&K.visible){const Q=v(A,K,T,M);A.onBeforeShadow(n,A,w,b,B,Q,N),n.renderBufferDirect(b,null,B,Q,A,N),A.onAfterShadow(n,A,w,b,B,Q,N)}}}else if(j.visible){const Y=v(A,j,T,M);A.onBeforeShadow(n,A,w,b,B,Y,null),n.renderBufferDirect(b,null,B,Y,A,null),A.onAfterShadow(n,A,w,b,B,Y,null)}}const V=A.children;for(let B=0,j=V.length;B<j;B++)x(V[B],w,b,T,M)}function R(A){A.target.removeEventListener("dispose",R);for(const b in c){const T=c[b],M=A.target.uuid;M in T&&(T[M].dispose(),delete T[M])}}}function lR(n){function e(){let D=!1;const re=new st;let X=null;const q=new st(0,0,0,0);return{setMask:function(ae){X!==ae&&!D&&(n.colorMask(ae,ae,ae,ae),X=ae)},setLocked:function(ae){D=ae},setClear:function(ae,Ae,Ye,Et,Ot){Ot===!0&&(ae*=Et,Ae*=Et,Ye*=Et),re.set(ae,Ae,Ye,Et),q.equals(re)===!1&&(n.clearColor(ae,Ae,Ye,Et),q.copy(re))},reset:function(){D=!1,X=null,q.set(-1,0,0,0)}}}function t(){let D=!1,re=null,X=null,q=null;return{setTest:function(ae){ae?he(n.DEPTH_TEST):de(n.DEPTH_TEST)},setMask:function(ae){re!==ae&&!D&&(n.depthMask(ae),re=ae)},setFunc:function(ae){if(X!==ae){switch(ae){case yM:n.depthFunc(n.NEVER);break;case xM:n.depthFunc(n.ALWAYS);break;case SM:n.depthFunc(n.LESS);break;case gc:n.depthFunc(n.LEQUAL);break;case MM:n.depthFunc(n.EQUAL);break;case EM:n.depthFunc(n.GEQUAL);break;case TM:n.depthFunc(n.GREATER);break;case wM:n.depthFunc(n.NOTEQUAL);break;default:n.depthFunc(n.LEQUAL)}X=ae}},setLocked:function(ae){D=ae},setClear:function(ae){q!==ae&&(n.clearDepth(ae),q=ae)},reset:function(){D=!1,re=null,X=null,q=null}}}function i(){let D=!1,re=null,X=null,q=null,ae=null,Ae=null,Ye=null,Et=null,Ot=null;return{setTest:function($e){D||($e?he(n.STENCIL_TEST):de(n.STENCIL_TEST))},setMask:function($e){re!==$e&&!D&&(n.stencilMask($e),re=$e)},setFunc:function($e,di,Zn){(X!==$e||q!==di||ae!==Zn)&&(n.stencilFunc($e,di,Zn),X=$e,q=di,ae=Zn)},setOp:function($e,di,Zn){(Ae!==$e||Ye!==di||Et!==Zn)&&(n.stencilOp($e,di,Zn),Ae=$e,Ye=di,Et=Zn)},setLocked:function($e){D=$e},setClear:function($e){Ot!==$e&&(n.clearStencil($e),Ot=$e)},reset:function(){D=!1,re=null,X=null,q=null,ae=null,Ae=null,Ye=null,Et=null,Ot=null}}}const r=new e,s=new t,o=new i,a=new WeakMap,l=new WeakMap;let c={},u={},d=new WeakMap,f=[],p=null,g=!1,y=null,m=null,h=null,_=null,v=null,x=null,R=null,A=new Pe(0,0,0),w=0,b=!1,T=null,M=null,I=null,V=null,B=null;const j=n.getParameter(n.MAX_COMBINED_TEXTURE_IMAGE_UNITS);let Y=!1,W=0;const $=n.getParameter(n.VERSION);$.indexOf("WebGL")!==-1?(W=parseFloat(/^WebGL (\d)/.exec($)[1]),Y=W>=1):$.indexOf("OpenGL ES")!==-1&&(W=parseFloat(/^OpenGL ES (\d)/.exec($)[1]),Y=W>=2);let N=null,K={};const Q=n.getParameter(n.SCISSOR_BOX),oe=n.getParameter(n.VIEWPORT),Me=new st().fromArray(Q),je=new st().fromArray(oe);function G(D,re,X,q){const ae=new Uint8Array(4),Ae=n.createTexture();n.bindTexture(D,Ae),n.texParameteri(D,n.TEXTURE_MIN_FILTER,n.NEAREST),n.texParameteri(D,n.TEXTURE_MAG_FILTER,n.NEAREST);for(let Ye=0;Ye<X;Ye++)D===n.TEXTURE_3D||D===n.TEXTURE_2D_ARRAY?n.texImage3D(re,0,n.RGBA,1,1,q,0,n.RGBA,n.UNSIGNED_BYTE,ae):n.texImage2D(re+Ye,0,n.RGBA,1,1,0,n.RGBA,n.UNSIGNED_BYTE,ae);return Ae}const ne={};ne[n.TEXTURE_2D]=G(n.TEXTURE_2D,n.TEXTURE_2D,1),ne[n.TEXTURE_CUBE_MAP]=G(n.TEXTURE_CUBE_MAP,n.TEXTURE_CUBE_MAP_POSITIVE_X,6),ne[n.TEXTURE_2D_ARRAY]=G(n.TEXTURE_2D_ARRAY,n.TEXTURE_2D_ARRAY,1,1),ne[n.TEXTURE_3D]=G(n.TEXTURE_3D,n.TEXTURE_3D,1,1),r.setClear(0,0,0,1),s.setClear(1),o.setClear(0),he(n.DEPTH_TEST),s.setFunc(gc),rt(!1),Ee(Em),he(n.CULL_FACE),St(cr);function he(D){c[D]!==!0&&(n.enable(D),c[D]=!0)}function de(D){c[D]!==!1&&(n.disable(D),c[D]=!1)}function Ce(D,re){return u[D]!==re?(n.bindFramebuffer(D,re),u[D]=re,D===n.DRAW_FRAMEBUFFER&&(u[n.FRAMEBUFFER]=re),D===n.FRAMEBUFFER&&(u[n.DRAW_FRAMEBUFFER]=re),!0):!1}function Fe(D,re){let X=f,q=!1;if(D){X=d.get(re),X===void 0&&(X=[],d.set(re,X));const ae=D.textures;if(X.length!==ae.length||X[0]!==n.COLOR_ATTACHMENT0){for(let Ae=0,Ye=ae.length;Ae<Ye;Ae++)X[Ae]=n.COLOR_ATTACHMENT0+Ae;X.length=ae.length,q=!0}}else X[0]!==n.BACK&&(X[0]=n.BACK,q=!0);q&&n.drawBuffers(X)}function We(D){return p!==D?(n.useProgram(D),p=D,!0):!1}const gt={[br]:n.FUNC_ADD,[nM]:n.FUNC_SUBTRACT,[iM]:n.FUNC_REVERSE_SUBTRACT};gt[rM]=n.MIN,gt[sM]=n.MAX;const P={[oM]:n.ZERO,[aM]:n.ONE,[lM]:n.SRC_COLOR,[nf]:n.SRC_ALPHA,[pM]:n.SRC_ALPHA_SATURATE,[fM]:n.DST_COLOR,[uM]:n.DST_ALPHA,[cM]:n.ONE_MINUS_SRC_COLOR,[rf]:n.ONE_MINUS_SRC_ALPHA,[hM]:n.ONE_MINUS_DST_COLOR,[dM]:n.ONE_MINUS_DST_ALPHA,[mM]:n.CONSTANT_COLOR,[gM]:n.ONE_MINUS_CONSTANT_COLOR,[_M]:n.CONSTANT_ALPHA,[vM]:n.ONE_MINUS_CONSTANT_ALPHA};function St(D,re,X,q,ae,Ae,Ye,Et,Ot,$e){if(D===cr){g===!0&&(de(n.BLEND),g=!1);return}if(g===!1&&(he(n.BLEND),g=!0),D!==tM){if(D!==y||$e!==b){if((m!==br||v!==br)&&(n.blendEquation(n.FUNC_ADD),m=br,v=br),$e)switch(D){case Ds:n.blendFuncSeparate(n.ONE,n.ONE_MINUS_SRC_ALPHA,n.ONE,n.ONE_MINUS_SRC_ALPHA);break;case mc:n.blendFunc(n.ONE,n.ONE);break;case Tm:n.blendFuncSeparate(n.ZERO,n.ONE_MINUS_SRC_COLOR,n.ZERO,n.ONE);break;case wm:n.blendFuncSeparate(n.ZERO,n.SRC_COLOR,n.ZERO,n.SRC_ALPHA);break;default:console.error("THREE.WebGLState: Invalid blending: ",D);break}else switch(D){case Ds:n.blendFuncSeparate(n.SRC_ALPHA,n.ONE_MINUS_SRC_ALPHA,n.ONE,n.ONE_MINUS_SRC_ALPHA);break;case mc:n.blendFunc(n.SRC_ALPHA,n.ONE);break;case Tm:n.blendFuncSeparate(n.ZERO,n.ONE_MINUS_SRC_COLOR,n.ZERO,n.ONE);break;case wm:n.blendFunc(n.ZERO,n.SRC_COLOR);break;default:console.error("THREE.WebGLState: Invalid blending: ",D);break}h=null,_=null,x=null,R=null,A.set(0,0,0),w=0,y=D,b=$e}return}ae=ae||re,Ae=Ae||X,Ye=Ye||q,(re!==m||ae!==v)&&(n.blendEquationSeparate(gt[re],gt[ae]),m=re,v=ae),(X!==h||q!==_||Ae!==x||Ye!==R)&&(n.blendFuncSeparate(P[X],P[q],P[Ae],P[Ye]),h=X,_=q,x=Ae,R=Ye),(Et.equals(A)===!1||Ot!==w)&&(n.blendColor(Et.r,Et.g,Et.b,Ot),A.copy(Et),w=Ot),y=D,b=!1}function et(D,re){D.side===ii?de(n.CULL_FACE):he(n.CULL_FACE);let X=D.side===fn;re&&(X=!X),rt(X),D.blending===Ds&&D.transparent===!1?St(cr):St(D.blending,D.blendEquation,D.blendSrc,D.blendDst,D.blendEquationAlpha,D.blendSrcAlpha,D.blendDstAlpha,D.blendColor,D.blendAlpha,D.premultipliedAlpha),s.setFunc(D.depthFunc),s.setTest(D.depthTest),s.setMask(D.depthWrite),r.setMask(D.colorWrite);const q=D.stencilWrite;o.setTest(q),q&&(o.setMask(D.stencilWriteMask),o.setFunc(D.stencilFunc,D.stencilRef,D.stencilFuncMask),o.setOp(D.stencilFail,D.stencilZFail,D.stencilZPass)),Ie(D.polygonOffset,D.polygonOffsetFactor,D.polygonOffsetUnits),D.alphaToCoverage===!0?he(n.SAMPLE_ALPHA_TO_COVERAGE):de(n.SAMPLE_ALPHA_TO_COVERAGE)}function rt(D){T!==D&&(D?n.frontFace(n.CW):n.frontFace(n.CCW),T=D)}function Ee(D){D!==QS?(he(n.CULL_FACE),D!==M&&(D===Em?n.cullFace(n.BACK):D===JS?n.cullFace(n.FRONT):n.cullFace(n.FRONT_AND_BACK))):de(n.CULL_FACE),M=D}function Mt(D){D!==I&&(Y&&n.lineWidth(D),I=D)}function Ie(D,re,X){D?(he(n.POLYGON_OFFSET_FILL),(V!==re||B!==X)&&(n.polygonOffset(re,X),V=re,B=X)):de(n.POLYGON_OFFSET_FILL)}function Oe(D){D?he(n.SCISSOR_TEST):de(n.SCISSOR_TEST)}function C(D){D===void 0&&(D=n.TEXTURE0+j-1),N!==D&&(n.activeTexture(D),N=D)}function S(D,re,X){X===void 0&&(N===null?X=n.TEXTURE0+j-1:X=N);let q=K[X];q===void 0&&(q={type:void 0,texture:void 0},K[X]=q),(q.type!==D||q.texture!==re)&&(N!==X&&(n.activeTexture(X),N=X),n.bindTexture(D,re||ne[D]),q.type=D,q.texture=re)}function z(){const D=K[N];D!==void 0&&D.type!==void 0&&(n.bindTexture(D.type,null),D.type=void 0,D.texture=void 0)}function Z(){try{n.compressedTexImage2D.apply(n,arguments)}catch(D){console.error("THREE.WebGLState:",D)}}function ee(){try{n.compressedTexImage3D.apply(n,arguments)}catch(D){console.error("THREE.WebGLState:",D)}}function J(){try{n.texSubImage2D.apply(n,arguments)}catch(D){console.error("THREE.WebGLState:",D)}}function Te(){try{n.texSubImage3D.apply(n,arguments)}catch(D){console.error("THREE.WebGLState:",D)}}function ce(){try{n.compressedTexSubImage2D.apply(n,arguments)}catch(D){console.error("THREE.WebGLState:",D)}}function ge(){try{n.compressedTexSubImage3D.apply(n,arguments)}catch(D){console.error("THREE.WebGLState:",D)}}function ke(){try{n.texStorage2D.apply(n,arguments)}catch(D){console.error("THREE.WebGLState:",D)}}function ie(){try{n.texStorage3D.apply(n,arguments)}catch(D){console.error("THREE.WebGLState:",D)}}function me(){try{n.texImage2D.apply(n,arguments)}catch(D){console.error("THREE.WebGLState:",D)}}function Xe(){try{n.texImage3D.apply(n,arguments)}catch(D){console.error("THREE.WebGLState:",D)}}function Le(D){Me.equals(D)===!1&&(n.scissor(D.x,D.y,D.z,D.w),Me.copy(D))}function _e(D){je.equals(D)===!1&&(n.viewport(D.x,D.y,D.z,D.w),je.copy(D))}function De(D,re){let X=l.get(re);X===void 0&&(X=new WeakMap,l.set(re,X));let q=X.get(D);q===void 0&&(q=n.getUniformBlockIndex(re,D.name),X.set(D,q))}function He(D,re){const q=l.get(re).get(D);a.get(re)!==q&&(n.uniformBlockBinding(re,q,D.__bindingPointIndex),a.set(re,q))}function ft(){n.disable(n.BLEND),n.disable(n.CULL_FACE),n.disable(n.DEPTH_TEST),n.disable(n.POLYGON_OFFSET_FILL),n.disable(n.SCISSOR_TEST),n.disable(n.STENCIL_TEST),n.disable(n.SAMPLE_ALPHA_TO_COVERAGE),n.blendEquation(n.FUNC_ADD),n.blendFunc(n.ONE,n.ZERO),n.blendFuncSeparate(n.ONE,n.ZERO,n.ONE,n.ZERO),n.blendColor(0,0,0,0),n.colorMask(!0,!0,!0,!0),n.clearColor(0,0,0,0),n.depthMask(!0),n.depthFunc(n.LESS),n.clearDepth(1),n.stencilMask(4294967295),n.stencilFunc(n.ALWAYS,0,4294967295),n.stencilOp(n.KEEP,n.KEEP,n.KEEP),n.clearStencil(0),n.cullFace(n.BACK),n.frontFace(n.CCW),n.polygonOffset(0,0),n.activeTexture(n.TEXTURE0),n.bindFramebuffer(n.FRAMEBUFFER,null),n.bindFramebuffer(n.DRAW_FRAMEBUFFER,null),n.bindFramebuffer(n.READ_FRAMEBUFFER,null),n.useProgram(null),n.lineWidth(1),n.scissor(0,0,n.canvas.width,n.canvas.height),n.viewport(0,0,n.canvas.width,n.canvas.height),c={},N=null,K={},u={},d=new WeakMap,f=[],p=null,g=!1,y=null,m=null,h=null,_=null,v=null,x=null,R=null,A=new Pe(0,0,0),w=0,b=!1,T=null,M=null,I=null,V=null,B=null,Me.set(0,0,n.canvas.width,n.canvas.height),je.set(0,0,n.canvas.width,n.canvas.height),r.reset(),s.reset(),o.reset()}return{buffers:{color:r,depth:s,stencil:o},enable:he,disable:de,bindFramebuffer:Ce,drawBuffers:Fe,useProgram:We,setBlending:St,setMaterial:et,setFlipSided:rt,setCullFace:Ee,setLineWidth:Mt,setPolygonOffset:Ie,setScissorTest:Oe,activeTexture:C,bindTexture:S,unbindTexture:z,compressedTexImage2D:Z,compressedTexImage3D:ee,texImage2D:me,texImage3D:Xe,updateUBOMapping:De,uniformBlockBinding:He,texStorage2D:ke,texStorage3D:ie,texSubImage2D:J,texSubImage3D:Te,compressedTexSubImage2D:ce,compressedTexSubImage3D:ge,scissor:Le,viewport:_e,reset:ft}}function Sg(n,e,t,i){const r=cR(i);switch(t){case Tv:return n*e;case Av:return n*e;case Rv:return n*e*2;case Hh:return n*e/r.components*r.byteLength;case Vh:return n*e/r.components*r.byteLength;case Cv:return n*e*2/r.components*r.byteLength;case Gh:return n*e*2/r.components*r.byteLength;case wv:return n*e*3/r.components*r.byteLength;case Pn:return n*e*4/r.components*r.byteLength;case Wh:return n*e*4/r.components*r.byteLength;case Bl:case zl:return Math.floor((n+3)/4)*Math.floor((e+3)/4)*8;case Hl:case Vl:return Math.floor((n+3)/4)*Math.floor((e+3)/4)*16;case lf:case uf:return Math.max(n,16)*Math.max(e,8)/4;case af:case cf:return Math.max(n,8)*Math.max(e,8)/2;case df:case ff:return Math.floor((n+3)/4)*Math.floor((e+3)/4)*8;case hf:return Math.floor((n+3)/4)*Math.floor((e+3)/4)*16;case pf:return Math.floor((n+3)/4)*Math.floor((e+3)/4)*16;case mf:return Math.floor((n+4)/5)*Math.floor((e+3)/4)*16;case gf:return Math.floor((n+4)/5)*Math.floor((e+4)/5)*16;case _f:return Math.floor((n+5)/6)*Math.floor((e+4)/5)*16;case vf:return Math.floor((n+5)/6)*Math.floor((e+5)/6)*16;case yf:return Math.floor((n+7)/8)*Math.floor((e+4)/5)*16;case xf:return Math.floor((n+7)/8)*Math.floor((e+5)/6)*16;case Sf:return Math.floor((n+7)/8)*Math.floor((e+7)/8)*16;case Mf:return Math.floor((n+9)/10)*Math.floor((e+4)/5)*16;case Ef:return Math.floor((n+9)/10)*Math.floor((e+5)/6)*16;case Tf:return Math.floor((n+9)/10)*Math.floor((e+7)/8)*16;case wf:return Math.floor((n+9)/10)*Math.floor((e+9)/10)*16;case Af:return Math.floor((n+11)/12)*Math.floor((e+9)/10)*16;case Rf:return Math.floor((n+11)/12)*Math.floor((e+11)/12)*16;case Gl:case Cf:case bf:return Math.ceil(n/4)*Math.ceil(e/4)*16;case bv:case Pf:return Math.ceil(n/4)*Math.ceil(e/4)*8;case Lf:case If:return Math.ceil(n/4)*Math.ceil(e/4)*16}throw new Error(`Unable to determine texture byte length for ${t} format.`)}function cR(n){switch(n){case Ni:case Sv:return{byteLength:1,components:1};case ha:case Mv:case Ta:return{byteLength:2,components:1};case Bh:case zh:return{byteLength:2,components:4};case Hr:case kh:case Xn:return{byteLength:4,components:1};case Ev:return{byteLength:4,components:3}}throw new Error(`Unknown texture type ${n}.`)}function uR(n,e,t,i,r,s,o){const a=e.has("WEBGL_multisampled_render_to_texture")?e.get("WEBGL_multisampled_render_to_texture"):null,l=typeof navigator>"u"?!1:/OculusBrowser/g.test(navigator.userAgent),c=new we,u=new WeakMap;let d;const f=new WeakMap;let p=!1;try{p=typeof OffscreenCanvas<"u"&&new OffscreenCanvas(1,1).getContext("2d")!==null}catch{}function g(C,S){return p?new OffscreenCanvas(C,S):ga("canvas")}function y(C,S,z){let Z=1;const ee=Oe(C);if((ee.width>z||ee.height>z)&&(Z=z/Math.max(ee.width,ee.height)),Z<1)if(typeof HTMLImageElement<"u"&&C instanceof HTMLImageElement||typeof HTMLCanvasElement<"u"&&C instanceof HTMLCanvasElement||typeof ImageBitmap<"u"&&C instanceof ImageBitmap||typeof VideoFrame<"u"&&C instanceof VideoFrame){const J=Math.floor(Z*ee.width),Te=Math.floor(Z*ee.height);d===void 0&&(d=g(J,Te));const ce=S?g(J,Te):d;return ce.width=J,ce.height=Te,ce.getContext("2d").drawImage(C,0,0,J,Te),console.warn("THREE.WebGLRenderer: Texture has been resized from ("+ee.width+"x"+ee.height+") to ("+J+"x"+Te+")."),ce}else return"data"in C&&console.warn("THREE.WebGLRenderer: Image in DataTexture is too big ("+ee.width+"x"+ee.height+")."),C;return C}function m(C){return C.generateMipmaps&&C.minFilter!==en&&C.minFilter!==an}function h(C){n.generateMipmap(C)}function _(C,S,z,Z,ee=!1){if(C!==null){if(n[C]!==void 0)return n[C];console.warn("THREE.WebGLRenderer: Attempt to use non-existing WebGL internal format '"+C+"'")}let J=S;if(S===n.RED&&(z===n.FLOAT&&(J=n.R32F),z===n.HALF_FLOAT&&(J=n.R16F),z===n.UNSIGNED_BYTE&&(J=n.R8)),S===n.RED_INTEGER&&(z===n.UNSIGNED_BYTE&&(J=n.R8UI),z===n.UNSIGNED_SHORT&&(J=n.R16UI),z===n.UNSIGNED_INT&&(J=n.R32UI),z===n.BYTE&&(J=n.R8I),z===n.SHORT&&(J=n.R16I),z===n.INT&&(J=n.R32I)),S===n.RG&&(z===n.FLOAT&&(J=n.RG32F),z===n.HALF_FLOAT&&(J=n.RG16F),z===n.UNSIGNED_BYTE&&(J=n.RG8)),S===n.RG_INTEGER&&(z===n.UNSIGNED_BYTE&&(J=n.RG8UI),z===n.UNSIGNED_SHORT&&(J=n.RG16UI),z===n.UNSIGNED_INT&&(J=n.RG32UI),z===n.BYTE&&(J=n.RG8I),z===n.SHORT&&(J=n.RG16I),z===n.INT&&(J=n.RG32I)),S===n.RGB&&z===n.UNSIGNED_INT_5_9_9_9_REV&&(J=n.RGB9_E5),S===n.RGBA){const Te=ee?vc:qe.getTransfer(Z);z===n.FLOAT&&(J=n.RGBA32F),z===n.HALF_FLOAT&&(J=n.RGBA16F),z===n.UNSIGNED_BYTE&&(J=Te===lt?n.SRGB8_ALPHA8:n.RGBA8),z===n.UNSIGNED_SHORT_4_4_4_4&&(J=n.RGBA4),z===n.UNSIGNED_SHORT_5_5_5_1&&(J=n.RGB5_A1)}return(J===n.R16F||J===n.R32F||J===n.RG16F||J===n.RG32F||J===n.RGBA16F||J===n.RGBA32F)&&e.get("EXT_color_buffer_float"),J}function v(C,S){let z;return C?S===null||S===Hr||S===qs?z=n.DEPTH24_STENCIL8:S===Xn?z=n.DEPTH32F_STENCIL8:S===ha&&(z=n.DEPTH24_STENCIL8,console.warn("DepthTexture: 16 bit depth attachment is not supported with stencil. Using 24-bit attachment.")):S===null||S===Hr||S===qs?z=n.DEPTH_COMPONENT24:S===Xn?z=n.DEPTH_COMPONENT32F:S===ha&&(z=n.DEPTH_COMPONENT16),z}function x(C,S){return m(C)===!0||C.isFramebufferTexture&&C.minFilter!==en&&C.minFilter!==an?Math.log2(Math.max(S.width,S.height))+1:C.mipmaps!==void 0&&C.mipmaps.length>0?C.mipmaps.length:C.isCompressedTexture&&Array.isArray(C.image)?S.mipmaps.length:1}function R(C){const S=C.target;S.removeEventListener("dispose",R),w(S),S.isVideoTexture&&u.delete(S)}function A(C){const S=C.target;S.removeEventListener("dispose",A),T(S)}function w(C){const S=i.get(C);if(S.__webglInit===void 0)return;const z=C.source,Z=f.get(z);if(Z){const ee=Z[S.__cacheKey];ee.usedTimes--,ee.usedTimes===0&&b(C),Object.keys(Z).length===0&&f.delete(z)}i.remove(C)}function b(C){const S=i.get(C);n.deleteTexture(S.__webglTexture);const z=C.source,Z=f.get(z);delete Z[S.__cacheKey],o.memory.textures--}function T(C){const S=i.get(C);if(C.depthTexture&&C.depthTexture.dispose(),C.isWebGLCubeRenderTarget)for(let Z=0;Z<6;Z++){if(Array.isArray(S.__webglFramebuffer[Z]))for(let ee=0;ee<S.__webglFramebuffer[Z].length;ee++)n.deleteFramebuffer(S.__webglFramebuffer[Z][ee]);else n.deleteFramebuffer(S.__webglFramebuffer[Z]);S.__webglDepthbuffer&&n.deleteRenderbuffer(S.__webglDepthbuffer[Z])}else{if(Array.isArray(S.__webglFramebuffer))for(let Z=0;Z<S.__webglFramebuffer.length;Z++)n.deleteFramebuffer(S.__webglFramebuffer[Z]);else n.deleteFramebuffer(S.__webglFramebuffer);if(S.__webglDepthbuffer&&n.deleteRenderbuffer(S.__webglDepthbuffer),S.__webglMultisampledFramebuffer&&n.deleteFramebuffer(S.__webglMultisampledFramebuffer),S.__webglColorRenderbuffer)for(let Z=0;Z<S.__webglColorRenderbuffer.length;Z++)S.__webglColorRenderbuffer[Z]&&n.deleteRenderbuffer(S.__webglColorRenderbuffer[Z]);S.__webglDepthRenderbuffer&&n.deleteRenderbuffer(S.__webglDepthRenderbuffer)}const z=C.textures;for(let Z=0,ee=z.length;Z<ee;Z++){const J=i.get(z[Z]);J.__webglTexture&&(n.deleteTexture(J.__webglTexture),o.memory.textures--),i.remove(z[Z])}i.remove(C)}let M=0;function I(){M=0}function V(){const C=M;return C>=r.maxTextures&&console.warn("THREE.WebGLTextures: Trying to use "+C+" texture units while this GPU supports only "+r.maxTextures),M+=1,C}function B(C){const S=[];return S.push(C.wrapS),S.push(C.wrapT),S.push(C.wrapR||0),S.push(C.magFilter),S.push(C.minFilter),S.push(C.anisotropy),S.push(C.internalFormat),S.push(C.format),S.push(C.type),S.push(C.generateMipmaps),S.push(C.premultiplyAlpha),S.push(C.flipY),S.push(C.unpackAlignment),S.push(C.colorSpace),S.join()}function j(C,S){const z=i.get(C);if(C.isVideoTexture&&Mt(C),C.isRenderTargetTexture===!1&&C.version>0&&z.__version!==C.version){const Z=C.image;if(Z===null)console.warn("THREE.WebGLRenderer: Texture marked for update but no image data found.");else if(Z.complete===!1)console.warn("THREE.WebGLRenderer: Texture marked for update but image is incomplete");else{je(z,C,S);return}}t.bindTexture(n.TEXTURE_2D,z.__webglTexture,n.TEXTURE0+S)}function Y(C,S){const z=i.get(C);if(C.version>0&&z.__version!==C.version){je(z,C,S);return}t.bindTexture(n.TEXTURE_2D_ARRAY,z.__webglTexture,n.TEXTURE0+S)}function W(C,S){const z=i.get(C);if(C.version>0&&z.__version!==C.version){je(z,C,S);return}t.bindTexture(n.TEXTURE_3D,z.__webglTexture,n.TEXTURE0+S)}function $(C,S){const z=i.get(C);if(C.version>0&&z.__version!==C.version){G(z,C,S);return}t.bindTexture(n.TEXTURE_CUBE_MAP,z.__webglTexture,n.TEXTURE0+S)}const N={[Ks]:n.REPEAT,[Ji]:n.CLAMP_TO_EDGE,[_c]:n.MIRRORED_REPEAT},K={[en]:n.NEAREST,[xv]:n.NEAREST_MIPMAP_NEAREST,[Fo]:n.NEAREST_MIPMAP_LINEAR,[an]:n.LINEAR,[kl]:n.LINEAR_MIPMAP_NEAREST,[Ti]:n.LINEAR_MIPMAP_LINEAR},Q={[HM]:n.NEVER,[YM]:n.ALWAYS,[VM]:n.LESS,[Iv]:n.LEQUAL,[GM]:n.EQUAL,[jM]:n.GEQUAL,[WM]:n.GREATER,[XM]:n.NOTEQUAL};function oe(C,S){if(S.type===Xn&&e.has("OES_texture_float_linear")===!1&&(S.magFilter===an||S.magFilter===kl||S.magFilter===Fo||S.magFilter===Ti||S.minFilter===an||S.minFilter===kl||S.minFilter===Fo||S.minFilter===Ti)&&console.warn("THREE.WebGLRenderer: Unable to use linear filtering with floating point textures. OES_texture_float_linear not supported on this device."),n.texParameteri(C,n.TEXTURE_WRAP_S,N[S.wrapS]),n.texParameteri(C,n.TEXTURE_WRAP_T,N[S.wrapT]),(C===n.TEXTURE_3D||C===n.TEXTURE_2D_ARRAY)&&n.texParameteri(C,n.TEXTURE_WRAP_R,N[S.wrapR]),n.texParameteri(C,n.TEXTURE_MAG_FILTER,K[S.magFilter]),n.texParameteri(C,n.TEXTURE_MIN_FILTER,K[S.minFilter]),S.compareFunction&&(n.texParameteri(C,n.TEXTURE_COMPARE_MODE,n.COMPARE_REF_TO_TEXTURE),n.texParameteri(C,n.TEXTURE_COMPARE_FUNC,Q[S.compareFunction])),e.has("EXT_texture_filter_anisotropic")===!0){if(S.magFilter===en||S.minFilter!==Fo&&S.minFilter!==Ti||S.type===Xn&&e.has("OES_texture_float_linear")===!1)return;if(S.anisotropy>1||i.get(S).__currentAnisotropy){const z=e.get("EXT_texture_filter_anisotropic");n.texParameterf(C,z.TEXTURE_MAX_ANISOTROPY_EXT,Math.min(S.anisotropy,r.getMaxAnisotropy())),i.get(S).__currentAnisotropy=S.anisotropy}}}function Me(C,S){let z=!1;C.__webglInit===void 0&&(C.__webglInit=!0,S.addEventListener("dispose",R));const Z=S.source;let ee=f.get(Z);ee===void 0&&(ee={},f.set(Z,ee));const J=B(S);if(J!==C.__cacheKey){ee[J]===void 0&&(ee[J]={texture:n.createTexture(),usedTimes:0},o.memory.textures++,z=!0),ee[J].usedTimes++;const Te=ee[C.__cacheKey];Te!==void 0&&(ee[C.__cacheKey].usedTimes--,Te.usedTimes===0&&b(S)),C.__cacheKey=J,C.__webglTexture=ee[J].texture}return z}function je(C,S,z){let Z=n.TEXTURE_2D;(S.isDataArrayTexture||S.isCompressedArrayTexture)&&(Z=n.TEXTURE_2D_ARRAY),S.isData3DTexture&&(Z=n.TEXTURE_3D);const ee=Me(C,S),J=S.source;t.bindTexture(Z,C.__webglTexture,n.TEXTURE0+z);const Te=i.get(J);if(J.version!==Te.__version||ee===!0){t.activeTexture(n.TEXTURE0+z);const ce=qe.getPrimaries(qe.workingColorSpace),ge=S.colorSpace===$i?null:qe.getPrimaries(S.colorSpace),ke=S.colorSpace===$i||ce===ge?n.NONE:n.BROWSER_DEFAULT_WEBGL;n.pixelStorei(n.UNPACK_FLIP_Y_WEBGL,S.flipY),n.pixelStorei(n.UNPACK_PREMULTIPLY_ALPHA_WEBGL,S.premultiplyAlpha),n.pixelStorei(n.UNPACK_ALIGNMENT,S.unpackAlignment),n.pixelStorei(n.UNPACK_COLORSPACE_CONVERSION_WEBGL,ke);let ie=y(S.image,!1,r.maxTextureSize);ie=Ie(S,ie);const me=s.convert(S.format,S.colorSpace),Xe=s.convert(S.type);let Le=_(S.internalFormat,me,Xe,S.colorSpace,S.isVideoTexture);oe(Z,S);let _e;const De=S.mipmaps,He=S.isVideoTexture!==!0,ft=Te.__version===void 0||ee===!0,D=J.dataReady,re=x(S,ie);if(S.isDepthTexture)Le=v(S.format===$s,S.type),ft&&(He?t.texStorage2D(n.TEXTURE_2D,1,Le,ie.width,ie.height):t.texImage2D(n.TEXTURE_2D,0,Le,ie.width,ie.height,0,me,Xe,null));else if(S.isDataTexture)if(De.length>0){He&&ft&&t.texStorage2D(n.TEXTURE_2D,re,Le,De[0].width,De[0].height);for(let X=0,q=De.length;X<q;X++)_e=De[X],He?D&&t.texSubImage2D(n.TEXTURE_2D,X,0,0,_e.width,_e.height,me,Xe,_e.data):t.texImage2D(n.TEXTURE_2D,X,Le,_e.width,_e.height,0,me,Xe,_e.data);S.generateMipmaps=!1}else He?(ft&&t.texStorage2D(n.TEXTURE_2D,re,Le,ie.width,ie.height),D&&t.texSubImage2D(n.TEXTURE_2D,0,0,0,ie.width,ie.height,me,Xe,ie.data)):t.texImage2D(n.TEXTURE_2D,0,Le,ie.width,ie.height,0,me,Xe,ie.data);else if(S.isCompressedTexture)if(S.isCompressedArrayTexture){He&&ft&&t.texStorage3D(n.TEXTURE_2D_ARRAY,re,Le,De[0].width,De[0].height,ie.depth);for(let X=0,q=De.length;X<q;X++)if(_e=De[X],S.format!==Pn)if(me!==null)if(He){if(D)if(S.layerUpdates.size>0){const ae=Sg(_e.width,_e.height,S.format,S.type);for(const Ae of S.layerUpdates){const Ye=_e.data.subarray(Ae*ae/_e.data.BYTES_PER_ELEMENT,(Ae+1)*ae/_e.data.BYTES_PER_ELEMENT);t.compressedTexSubImage3D(n.TEXTURE_2D_ARRAY,X,0,0,Ae,_e.width,_e.height,1,me,Ye,0,0)}S.clearLayerUpdates()}else t.compressedTexSubImage3D(n.TEXTURE_2D_ARRAY,X,0,0,0,_e.width,_e.height,ie.depth,me,_e.data,0,0)}else t.compressedTexImage3D(n.TEXTURE_2D_ARRAY,X,Le,_e.width,_e.height,ie.depth,0,_e.data,0,0);else console.warn("THREE.WebGLRenderer: Attempt to load unsupported compressed texture format in .uploadTexture()");else He?D&&t.texSubImage3D(n.TEXTURE_2D_ARRAY,X,0,0,0,_e.width,_e.height,ie.depth,me,Xe,_e.data):t.texImage3D(n.TEXTURE_2D_ARRAY,X,Le,_e.width,_e.height,ie.depth,0,me,Xe,_e.data)}else{He&&ft&&t.texStorage2D(n.TEXTURE_2D,re,Le,De[0].width,De[0].height);for(let X=0,q=De.length;X<q;X++)_e=De[X],S.format!==Pn?me!==null?He?D&&t.compressedTexSubImage2D(n.TEXTURE_2D,X,0,0,_e.width,_e.height,me,_e.data):t.compressedTexImage2D(n.TEXTURE_2D,X,Le,_e.width,_e.height,0,_e.data):console.warn("THREE.WebGLRenderer: Attempt to load unsupported compressed texture format in .uploadTexture()"):He?D&&t.texSubImage2D(n.TEXTURE_2D,X,0,0,_e.width,_e.height,me,Xe,_e.data):t.texImage2D(n.TEXTURE_2D,X,Le,_e.width,_e.height,0,me,Xe,_e.data)}else if(S.isDataArrayTexture)if(He){if(ft&&t.texStorage3D(n.TEXTURE_2D_ARRAY,re,Le,ie.width,ie.height,ie.depth),D)if(S.layerUpdates.size>0){const X=Sg(ie.width,ie.height,S.format,S.type);for(const q of S.layerUpdates){const ae=ie.data.subarray(q*X/ie.data.BYTES_PER_ELEMENT,(q+1)*X/ie.data.BYTES_PER_ELEMENT);t.texSubImage3D(n.TEXTURE_2D_ARRAY,0,0,0,q,ie.width,ie.height,1,me,Xe,ae)}S.clearLayerUpdates()}else t.texSubImage3D(n.TEXTURE_2D_ARRAY,0,0,0,0,ie.width,ie.height,ie.depth,me,Xe,ie.data)}else t.texImage3D(n.TEXTURE_2D_ARRAY,0,Le,ie.width,ie.height,ie.depth,0,me,Xe,ie.data);else if(S.isData3DTexture)He?(ft&&t.texStorage3D(n.TEXTURE_3D,re,Le,ie.width,ie.height,ie.depth),D&&t.texSubImage3D(n.TEXTURE_3D,0,0,0,0,ie.width,ie.height,ie.depth,me,Xe,ie.data)):t.texImage3D(n.TEXTURE_3D,0,Le,ie.width,ie.height,ie.depth,0,me,Xe,ie.data);else if(S.isFramebufferTexture){if(ft)if(He)t.texStorage2D(n.TEXTURE_2D,re,Le,ie.width,ie.height);else{let X=ie.width,q=ie.height;for(let ae=0;ae<re;ae++)t.texImage2D(n.TEXTURE_2D,ae,Le,X,q,0,me,Xe,null),X>>=1,q>>=1}}else if(De.length>0){if(He&&ft){const X=Oe(De[0]);t.texStorage2D(n.TEXTURE_2D,re,Le,X.width,X.height)}for(let X=0,q=De.length;X<q;X++)_e=De[X],He?D&&t.texSubImage2D(n.TEXTURE_2D,X,0,0,me,Xe,_e):t.texImage2D(n.TEXTURE_2D,X,Le,me,Xe,_e);S.generateMipmaps=!1}else if(He){if(ft){const X=Oe(ie);t.texStorage2D(n.TEXTURE_2D,re,Le,X.width,X.height)}D&&t.texSubImage2D(n.TEXTURE_2D,0,0,0,me,Xe,ie)}else t.texImage2D(n.TEXTURE_2D,0,Le,me,Xe,ie);m(S)&&h(Z),Te.__version=J.version,S.onUpdate&&S.onUpdate(S)}C.__version=S.version}function G(C,S,z){if(S.image.length!==6)return;const Z=Me(C,S),ee=S.source;t.bindTexture(n.TEXTURE_CUBE_MAP,C.__webglTexture,n.TEXTURE0+z);const J=i.get(ee);if(ee.version!==J.__version||Z===!0){t.activeTexture(n.TEXTURE0+z);const Te=qe.getPrimaries(qe.workingColorSpace),ce=S.colorSpace===$i?null:qe.getPrimaries(S.colorSpace),ge=S.colorSpace===$i||Te===ce?n.NONE:n.BROWSER_DEFAULT_WEBGL;n.pixelStorei(n.UNPACK_FLIP_Y_WEBGL,S.flipY),n.pixelStorei(n.UNPACK_PREMULTIPLY_ALPHA_WEBGL,S.premultiplyAlpha),n.pixelStorei(n.UNPACK_ALIGNMENT,S.unpackAlignment),n.pixelStorei(n.UNPACK_COLORSPACE_CONVERSION_WEBGL,ge);const ke=S.isCompressedTexture||S.image[0].isCompressedTexture,ie=S.image[0]&&S.image[0].isDataTexture,me=[];for(let q=0;q<6;q++)!ke&&!ie?me[q]=y(S.image[q],!0,r.maxCubemapSize):me[q]=ie?S.image[q].image:S.image[q],me[q]=Ie(S,me[q]);const Xe=me[0],Le=s.convert(S.format,S.colorSpace),_e=s.convert(S.type),De=_(S.internalFormat,Le,_e,S.colorSpace),He=S.isVideoTexture!==!0,ft=J.__version===void 0||Z===!0,D=ee.dataReady;let re=x(S,Xe);oe(n.TEXTURE_CUBE_MAP,S);let X;if(ke){He&&ft&&t.texStorage2D(n.TEXTURE_CUBE_MAP,re,De,Xe.width,Xe.height);for(let q=0;q<6;q++){X=me[q].mipmaps;for(let ae=0;ae<X.length;ae++){const Ae=X[ae];S.format!==Pn?Le!==null?He?D&&t.compressedTexSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,ae,0,0,Ae.width,Ae.height,Le,Ae.data):t.compressedTexImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,ae,De,Ae.width,Ae.height,0,Ae.data):console.warn("THREE.WebGLRenderer: Attempt to load unsupported compressed texture format in .setTextureCube()"):He?D&&t.texSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,ae,0,0,Ae.width,Ae.height,Le,_e,Ae.data):t.texImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,ae,De,Ae.width,Ae.height,0,Le,_e,Ae.data)}}}else{if(X=S.mipmaps,He&&ft){X.length>0&&re++;const q=Oe(me[0]);t.texStorage2D(n.TEXTURE_CUBE_MAP,re,De,q.width,q.height)}for(let q=0;q<6;q++)if(ie){He?D&&t.texSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,0,0,0,me[q].width,me[q].height,Le,_e,me[q].data):t.texImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,0,De,me[q].width,me[q].height,0,Le,_e,me[q].data);for(let ae=0;ae<X.length;ae++){const Ye=X[ae].image[q].image;He?D&&t.texSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,ae+1,0,0,Ye.width,Ye.height,Le,_e,Ye.data):t.texImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,ae+1,De,Ye.width,Ye.height,0,Le,_e,Ye.data)}}else{He?D&&t.texSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,0,0,0,Le,_e,me[q]):t.texImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,0,De,Le,_e,me[q]);for(let ae=0;ae<X.length;ae++){const Ae=X[ae];He?D&&t.texSubImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,ae+1,0,0,Le,_e,Ae.image[q]):t.texImage2D(n.TEXTURE_CUBE_MAP_POSITIVE_X+q,ae+1,De,Le,_e,Ae.image[q])}}}m(S)&&h(n.TEXTURE_CUBE_MAP),J.__version=ee.version,S.onUpdate&&S.onUpdate(S)}C.__version=S.version}function ne(C,S,z,Z,ee,J){const Te=s.convert(z.format,z.colorSpace),ce=s.convert(z.type),ge=_(z.internalFormat,Te,ce,z.colorSpace);if(!i.get(S).__hasExternalTextures){const ie=Math.max(1,S.width>>J),me=Math.max(1,S.height>>J);ee===n.TEXTURE_3D||ee===n.TEXTURE_2D_ARRAY?t.texImage3D(ee,J,ge,ie,me,S.depth,0,Te,ce,null):t.texImage2D(ee,J,ge,ie,me,0,Te,ce,null)}t.bindFramebuffer(n.FRAMEBUFFER,C),Ee(S)?a.framebufferTexture2DMultisampleEXT(n.FRAMEBUFFER,Z,ee,i.get(z).__webglTexture,0,rt(S)):(ee===n.TEXTURE_2D||ee>=n.TEXTURE_CUBE_MAP_POSITIVE_X&&ee<=n.TEXTURE_CUBE_MAP_NEGATIVE_Z)&&n.framebufferTexture2D(n.FRAMEBUFFER,Z,ee,i.get(z).__webglTexture,J),t.bindFramebuffer(n.FRAMEBUFFER,null)}function he(C,S,z){if(n.bindRenderbuffer(n.RENDERBUFFER,C),S.depthBuffer){const Z=S.depthTexture,ee=Z&&Z.isDepthTexture?Z.type:null,J=v(S.stencilBuffer,ee),Te=S.stencilBuffer?n.DEPTH_STENCIL_ATTACHMENT:n.DEPTH_ATTACHMENT,ce=rt(S);Ee(S)?a.renderbufferStorageMultisampleEXT(n.RENDERBUFFER,ce,J,S.width,S.height):z?n.renderbufferStorageMultisample(n.RENDERBUFFER,ce,J,S.width,S.height):n.renderbufferStorage(n.RENDERBUFFER,J,S.width,S.height),n.framebufferRenderbuffer(n.FRAMEBUFFER,Te,n.RENDERBUFFER,C)}else{const Z=S.textures;for(let ee=0;ee<Z.length;ee++){const J=Z[ee],Te=s.convert(J.format,J.colorSpace),ce=s.convert(J.type),ge=_(J.internalFormat,Te,ce,J.colorSpace),ke=rt(S);z&&Ee(S)===!1?n.renderbufferStorageMultisample(n.RENDERBUFFER,ke,ge,S.width,S.height):Ee(S)?a.renderbufferStorageMultisampleEXT(n.RENDERBUFFER,ke,ge,S.width,S.height):n.renderbufferStorage(n.RENDERBUFFER,ge,S.width,S.height)}}n.bindRenderbuffer(n.RENDERBUFFER,null)}function de(C,S){if(S&&S.isWebGLCubeRenderTarget)throw new Error("Depth Texture with cube render targets is not supported");if(t.bindFramebuffer(n.FRAMEBUFFER,C),!(S.depthTexture&&S.depthTexture.isDepthTexture))throw new Error("renderTarget.depthTexture must be an instance of THREE.DepthTexture");(!i.get(S.depthTexture).__webglTexture||S.depthTexture.image.width!==S.width||S.depthTexture.image.height!==S.height)&&(S.depthTexture.image.width=S.width,S.depthTexture.image.height=S.height,S.depthTexture.needsUpdate=!0),j(S.depthTexture,0);const Z=i.get(S.depthTexture).__webglTexture,ee=rt(S);if(S.depthTexture.format===Us)Ee(S)?a.framebufferTexture2DMultisampleEXT(n.FRAMEBUFFER,n.DEPTH_ATTACHMENT,n.TEXTURE_2D,Z,0,ee):n.framebufferTexture2D(n.FRAMEBUFFER,n.DEPTH_ATTACHMENT,n.TEXTURE_2D,Z,0);else if(S.depthTexture.format===$s)Ee(S)?a.framebufferTexture2DMultisampleEXT(n.FRAMEBUFFER,n.DEPTH_STENCIL_ATTACHMENT,n.TEXTURE_2D,Z,0,ee):n.framebufferTexture2D(n.FRAMEBUFFER,n.DEPTH_STENCIL_ATTACHMENT,n.TEXTURE_2D,Z,0);else throw new Error("Unknown depthTexture format")}function Ce(C){const S=i.get(C),z=C.isWebGLCubeRenderTarget===!0;if(S.__boundDepthTexture!==C.depthTexture){const Z=C.depthTexture;if(S.__depthDisposeCallback&&S.__depthDisposeCallback(),Z){const ee=()=>{delete S.__boundDepthTexture,delete S.__depthDisposeCallback,Z.removeEventListener("dispose",ee)};Z.addEventListener("dispose",ee),S.__depthDisposeCallback=ee}S.__boundDepthTexture=Z}if(C.depthTexture&&!S.__autoAllocateDepthBuffer){if(z)throw new Error("target.depthTexture not supported in Cube render targets");de(S.__webglFramebuffer,C)}else if(z){S.__webglDepthbuffer=[];for(let Z=0;Z<6;Z++)if(t.bindFramebuffer(n.FRAMEBUFFER,S.__webglFramebuffer[Z]),S.__webglDepthbuffer[Z]===void 0)S.__webglDepthbuffer[Z]=n.createRenderbuffer(),he(S.__webglDepthbuffer[Z],C,!1);else{const ee=C.stencilBuffer?n.DEPTH_STENCIL_ATTACHMENT:n.DEPTH_ATTACHMENT,J=S.__webglDepthbuffer[Z];n.bindRenderbuffer(n.RENDERBUFFER,J),n.framebufferRenderbuffer(n.FRAMEBUFFER,ee,n.RENDERBUFFER,J)}}else if(t.bindFramebuffer(n.FRAMEBUFFER,S.__webglFramebuffer),S.__webglDepthbuffer===void 0)S.__webglDepthbuffer=n.createRenderbuffer(),he(S.__webglDepthbuffer,C,!1);else{const Z=C.stencilBuffer?n.DEPTH_STENCIL_ATTACHMENT:n.DEPTH_ATTACHMENT,ee=S.__webglDepthbuffer;n.bindRenderbuffer(n.RENDERBUFFER,ee),n.framebufferRenderbuffer(n.FRAMEBUFFER,Z,n.RENDERBUFFER,ee)}t.bindFramebuffer(n.FRAMEBUFFER,null)}function Fe(C,S,z){const Z=i.get(C);S!==void 0&&ne(Z.__webglFramebuffer,C,C.texture,n.COLOR_ATTACHMENT0,n.TEXTURE_2D,0),z!==void 0&&Ce(C)}function We(C){const S=C.texture,z=i.get(C),Z=i.get(S);C.addEventListener("dispose",A);const ee=C.textures,J=C.isWebGLCubeRenderTarget===!0,Te=ee.length>1;if(Te||(Z.__webglTexture===void 0&&(Z.__webglTexture=n.createTexture()),Z.__version=S.version,o.memory.textures++),J){z.__webglFramebuffer=[];for(let ce=0;ce<6;ce++)if(S.mipmaps&&S.mipmaps.length>0){z.__webglFramebuffer[ce]=[];for(let ge=0;ge<S.mipmaps.length;ge++)z.__webglFramebuffer[ce][ge]=n.createFramebuffer()}else z.__webglFramebuffer[ce]=n.createFramebuffer()}else{if(S.mipmaps&&S.mipmaps.length>0){z.__webglFramebuffer=[];for(let ce=0;ce<S.mipmaps.length;ce++)z.__webglFramebuffer[ce]=n.createFramebuffer()}else z.__webglFramebuffer=n.createFramebuffer();if(Te)for(let ce=0,ge=ee.length;ce<ge;ce++){const ke=i.get(ee[ce]);ke.__webglTexture===void 0&&(ke.__webglTexture=n.createTexture(),o.memory.textures++)}if(C.samples>0&&Ee(C)===!1){z.__webglMultisampledFramebuffer=n.createFramebuffer(),z.__webglColorRenderbuffer=[],t.bindFramebuffer(n.FRAMEBUFFER,z.__webglMultisampledFramebuffer);for(let ce=0;ce<ee.length;ce++){const ge=ee[ce];z.__webglColorRenderbuffer[ce]=n.createRenderbuffer(),n.bindRenderbuffer(n.RENDERBUFFER,z.__webglColorRenderbuffer[ce]);const ke=s.convert(ge.format,ge.colorSpace),ie=s.convert(ge.type),me=_(ge.internalFormat,ke,ie,ge.colorSpace,C.isXRRenderTarget===!0),Xe=rt(C);n.renderbufferStorageMultisample(n.RENDERBUFFER,Xe,me,C.width,C.height),n.framebufferRenderbuffer(n.FRAMEBUFFER,n.COLOR_ATTACHMENT0+ce,n.RENDERBUFFER,z.__webglColorRenderbuffer[ce])}n.bindRenderbuffer(n.RENDERBUFFER,null),C.depthBuffer&&(z.__webglDepthRenderbuffer=n.createRenderbuffer(),he(z.__webglDepthRenderbuffer,C,!0)),t.bindFramebuffer(n.FRAMEBUFFER,null)}}if(J){t.bindTexture(n.TEXTURE_CUBE_MAP,Z.__webglTexture),oe(n.TEXTURE_CUBE_MAP,S);for(let ce=0;ce<6;ce++)if(S.mipmaps&&S.mipmaps.length>0)for(let ge=0;ge<S.mipmaps.length;ge++)ne(z.__webglFramebuffer[ce][ge],C,S,n.COLOR_ATTACHMENT0,n.TEXTURE_CUBE_MAP_POSITIVE_X+ce,ge);else ne(z.__webglFramebuffer[ce],C,S,n.COLOR_ATTACHMENT0,n.TEXTURE_CUBE_MAP_POSITIVE_X+ce,0);m(S)&&h(n.TEXTURE_CUBE_MAP),t.unbindTexture()}else if(Te){for(let ce=0,ge=ee.length;ce<ge;ce++){const ke=ee[ce],ie=i.get(ke);t.bindTexture(n.TEXTURE_2D,ie.__webglTexture),oe(n.TEXTURE_2D,ke),ne(z.__webglFramebuffer,C,ke,n.COLOR_ATTACHMENT0+ce,n.TEXTURE_2D,0),m(ke)&&h(n.TEXTURE_2D)}t.unbindTexture()}else{let ce=n.TEXTURE_2D;if((C.isWebGL3DRenderTarget||C.isWebGLArrayRenderTarget)&&(ce=C.isWebGL3DRenderTarget?n.TEXTURE_3D:n.TEXTURE_2D_ARRAY),t.bindTexture(ce,Z.__webglTexture),oe(ce,S),S.mipmaps&&S.mipmaps.length>0)for(let ge=0;ge<S.mipmaps.length;ge++)ne(z.__webglFramebuffer[ge],C,S,n.COLOR_ATTACHMENT0,ce,ge);else ne(z.__webglFramebuffer,C,S,n.COLOR_ATTACHMENT0,ce,0);m(S)&&h(ce),t.unbindTexture()}C.depthBuffer&&Ce(C)}function gt(C){const S=C.textures;for(let z=0,Z=S.length;z<Z;z++){const ee=S[z];if(m(ee)){const J=C.isWebGLCubeRenderTarget?n.TEXTURE_CUBE_MAP:n.TEXTURE_2D,Te=i.get(ee).__webglTexture;t.bindTexture(J,Te),h(J),t.unbindTexture()}}}const P=[],St=[];function et(C){if(C.samples>0){if(Ee(C)===!1){const S=C.textures,z=C.width,Z=C.height;let ee=n.COLOR_BUFFER_BIT;const J=C.stencilBuffer?n.DEPTH_STENCIL_ATTACHMENT:n.DEPTH_ATTACHMENT,Te=i.get(C),ce=S.length>1;if(ce)for(let ge=0;ge<S.length;ge++)t.bindFramebuffer(n.FRAMEBUFFER,Te.__webglMultisampledFramebuffer),n.framebufferRenderbuffer(n.FRAMEBUFFER,n.COLOR_ATTACHMENT0+ge,n.RENDERBUFFER,null),t.bindFramebuffer(n.FRAMEBUFFER,Te.__webglFramebuffer),n.framebufferTexture2D(n.DRAW_FRAMEBUFFER,n.COLOR_ATTACHMENT0+ge,n.TEXTURE_2D,null,0);t.bindFramebuffer(n.READ_FRAMEBUFFER,Te.__webglMultisampledFramebuffer),t.bindFramebuffer(n.DRAW_FRAMEBUFFER,Te.__webglFramebuffer);for(let ge=0;ge<S.length;ge++){if(C.resolveDepthBuffer&&(C.depthBuffer&&(ee|=n.DEPTH_BUFFER_BIT),C.stencilBuffer&&C.resolveStencilBuffer&&(ee|=n.STENCIL_BUFFER_BIT)),ce){n.framebufferRenderbuffer(n.READ_FRAMEBUFFER,n.COLOR_ATTACHMENT0,n.RENDERBUFFER,Te.__webglColorRenderbuffer[ge]);const ke=i.get(S[ge]).__webglTexture;n.framebufferTexture2D(n.DRAW_FRAMEBUFFER,n.COLOR_ATTACHMENT0,n.TEXTURE_2D,ke,0)}n.blitFramebuffer(0,0,z,Z,0,0,z,Z,ee,n.NEAREST),l===!0&&(P.length=0,St.length=0,P.push(n.COLOR_ATTACHMENT0+ge),C.depthBuffer&&C.resolveDepthBuffer===!1&&(P.push(J),St.push(J),n.invalidateFramebuffer(n.DRAW_FRAMEBUFFER,St)),n.invalidateFramebuffer(n.READ_FRAMEBUFFER,P))}if(t.bindFramebuffer(n.READ_FRAMEBUFFER,null),t.bindFramebuffer(n.DRAW_FRAMEBUFFER,null),ce)for(let ge=0;ge<S.length;ge++){t.bindFramebuffer(n.FRAMEBUFFER,Te.__webglMultisampledFramebuffer),n.framebufferRenderbuffer(n.FRAMEBUFFER,n.COLOR_ATTACHMENT0+ge,n.RENDERBUFFER,Te.__webglColorRenderbuffer[ge]);const ke=i.get(S[ge]).__webglTexture;t.bindFramebuffer(n.FRAMEBUFFER,Te.__webglFramebuffer),n.framebufferTexture2D(n.DRAW_FRAMEBUFFER,n.COLOR_ATTACHMENT0+ge,n.TEXTURE_2D,ke,0)}t.bindFramebuffer(n.DRAW_FRAMEBUFFER,Te.__webglMultisampledFramebuffer)}else if(C.depthBuffer&&C.resolveDepthBuffer===!1&&l){const S=C.stencilBuffer?n.DEPTH_STENCIL_ATTACHMENT:n.DEPTH_ATTACHMENT;n.invalidateFramebuffer(n.DRAW_FRAMEBUFFER,[S])}}}function rt(C){return Math.min(r.maxSamples,C.samples)}function Ee(C){const S=i.get(C);return C.samples>0&&e.has("WEBGL_multisampled_render_to_texture")===!0&&S.__useRenderToTexture!==!1}function Mt(C){const S=o.render.frame;u.get(C)!==S&&(u.set(C,S),C.update())}function Ie(C,S){const z=C.colorSpace,Z=C.format,ee=C.type;return C.isCompressedTexture===!0||C.isVideoTexture===!0||z!==Vt&&z!==$i&&(qe.getTransfer(z)===lt?(Z!==Pn||ee!==Ni)&&console.warn("THREE.WebGLTextures: sRGB encoded textures have to use RGBAFormat and UnsignedByteType."):console.error("THREE.WebGLTextures: Unsupported texture color space:",z)),S}function Oe(C){return typeof HTMLImageElement<"u"&&C instanceof HTMLImageElement?(c.width=C.naturalWidth||C.width,c.height=C.naturalHeight||C.height):typeof VideoFrame<"u"&&C instanceof VideoFrame?(c.width=C.displayWidth,c.height=C.displayHeight):(c.width=C.width,c.height=C.height),c}this.allocateTextureUnit=V,this.resetTextureUnits=I,this.setTexture2D=j,this.setTexture2DArray=Y,this.setTexture3D=W,this.setTextureCube=$,this.rebindTextures=Fe,this.setupRenderTarget=We,this.updateRenderTargetMipmap=gt,this.updateMultisampleRenderTarget=et,this.setupDepthRenderbuffer=Ce,this.setupFrameBufferTexture=ne,this.useMultisampledRTT=Ee}function dR(n,e){function t(i,r=$i){let s;const o=qe.getTransfer(r);if(i===Ni)return n.UNSIGNED_BYTE;if(i===Bh)return n.UNSIGNED_SHORT_4_4_4_4;if(i===zh)return n.UNSIGNED_SHORT_5_5_5_1;if(i===Ev)return n.UNSIGNED_INT_5_9_9_9_REV;if(i===Sv)return n.BYTE;if(i===Mv)return n.SHORT;if(i===ha)return n.UNSIGNED_SHORT;if(i===kh)return n.INT;if(i===Hr)return n.UNSIGNED_INT;if(i===Xn)return n.FLOAT;if(i===Ta)return n.HALF_FLOAT;if(i===Tv)return n.ALPHA;if(i===wv)return n.RGB;if(i===Pn)return n.RGBA;if(i===Av)return n.LUMINANCE;if(i===Rv)return n.LUMINANCE_ALPHA;if(i===Us)return n.DEPTH_COMPONENT;if(i===$s)return n.DEPTH_STENCIL;if(i===Hh)return n.RED;if(i===Vh)return n.RED_INTEGER;if(i===Cv)return n.RG;if(i===Gh)return n.RG_INTEGER;if(i===Wh)return n.RGBA_INTEGER;if(i===Bl||i===zl||i===Hl||i===Vl)if(o===lt)if(s=e.get("WEBGL_compressed_texture_s3tc_srgb"),s!==null){if(i===Bl)return s.COMPRESSED_SRGB_S3TC_DXT1_EXT;if(i===zl)return s.COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT;if(i===Hl)return s.COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT;if(i===Vl)return s.COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT}else return null;else if(s=e.get("WEBGL_compressed_texture_s3tc"),s!==null){if(i===Bl)return s.COMPRESSED_RGB_S3TC_DXT1_EXT;if(i===zl)return s.COMPRESSED_RGBA_S3TC_DXT1_EXT;if(i===Hl)return s.COMPRESSED_RGBA_S3TC_DXT3_EXT;if(i===Vl)return s.COMPRESSED_RGBA_S3TC_DXT5_EXT}else return null;if(i===af||i===lf||i===cf||i===uf)if(s=e.get("WEBGL_compressed_texture_pvrtc"),s!==null){if(i===af)return s.COMPRESSED_RGB_PVRTC_4BPPV1_IMG;if(i===lf)return s.COMPRESSED_RGB_PVRTC_2BPPV1_IMG;if(i===cf)return s.COMPRESSED_RGBA_PVRTC_4BPPV1_IMG;if(i===uf)return s.COMPRESSED_RGBA_PVRTC_2BPPV1_IMG}else return null;if(i===df||i===ff||i===hf)if(s=e.get("WEBGL_compressed_texture_etc"),s!==null){if(i===df||i===ff)return o===lt?s.COMPRESSED_SRGB8_ETC2:s.COMPRESSED_RGB8_ETC2;if(i===hf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ETC2_EAC:s.COMPRESSED_RGBA8_ETC2_EAC}else return null;if(i===pf||i===mf||i===gf||i===_f||i===vf||i===yf||i===xf||i===Sf||i===Mf||i===Ef||i===Tf||i===wf||i===Af||i===Rf)if(s=e.get("WEBGL_compressed_texture_astc"),s!==null){if(i===pf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_4x4_KHR:s.COMPRESSED_RGBA_ASTC_4x4_KHR;if(i===mf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_5x4_KHR:s.COMPRESSED_RGBA_ASTC_5x4_KHR;if(i===gf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_5x5_KHR:s.COMPRESSED_RGBA_ASTC_5x5_KHR;if(i===_f)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_6x5_KHR:s.COMPRESSED_RGBA_ASTC_6x5_KHR;if(i===vf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_6x6_KHR:s.COMPRESSED_RGBA_ASTC_6x6_KHR;if(i===yf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_8x5_KHR:s.COMPRESSED_RGBA_ASTC_8x5_KHR;if(i===xf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_8x6_KHR:s.COMPRESSED_RGBA_ASTC_8x6_KHR;if(i===Sf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_8x8_KHR:s.COMPRESSED_RGBA_ASTC_8x8_KHR;if(i===Mf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_10x5_KHR:s.COMPRESSED_RGBA_ASTC_10x5_KHR;if(i===Ef)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_10x6_KHR:s.COMPRESSED_RGBA_ASTC_10x6_KHR;if(i===Tf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_10x8_KHR:s.COMPRESSED_RGBA_ASTC_10x8_KHR;if(i===wf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_10x10_KHR:s.COMPRESSED_RGBA_ASTC_10x10_KHR;if(i===Af)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_12x10_KHR:s.COMPRESSED_RGBA_ASTC_12x10_KHR;if(i===Rf)return o===lt?s.COMPRESSED_SRGB8_ALPHA8_ASTC_12x12_KHR:s.COMPRESSED_RGBA_ASTC_12x12_KHR}else return null;if(i===Gl||i===Cf||i===bf)if(s=e.get("EXT_texture_compression_bptc"),s!==null){if(i===Gl)return o===lt?s.COMPRESSED_SRGB_ALPHA_BPTC_UNORM_EXT:s.COMPRESSED_RGBA_BPTC_UNORM_EXT;if(i===Cf)return s.COMPRESSED_RGB_BPTC_SIGNED_FLOAT_EXT;if(i===bf)return s.COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT_EXT}else return null;if(i===bv||i===Pf||i===Lf||i===If)if(s=e.get("EXT_texture_compression_rgtc"),s!==null){if(i===Gl)return s.COMPRESSED_RED_RGTC1_EXT;if(i===Pf)return s.COMPRESSED_SIGNED_RED_RGTC1_EXT;if(i===Lf)return s.COMPRESSED_RED_GREEN_RGTC2_EXT;if(i===If)return s.COMPRESSED_SIGNED_RED_GREEN_RGTC2_EXT}else return null;return i===qs?n.UNSIGNED_INT_24_8:n[i]!==void 0?n[i]:null}return{convert:t}}class fR extends Jt{constructor(e=[]){super(),this.isArrayCamera=!0,this.cameras=e}}class ri extends dt{constructor(){super(),this.isGroup=!0,this.type="Group"}}const hR={type:"move"};class ju{constructor(){this._targetRay=null,this._grip=null,this._hand=null}getHandSpace(){return this._hand===null&&(this._hand=new ri,this._hand.matrixAutoUpdate=!1,this._hand.visible=!1,this._hand.joints={},this._hand.inputState={pinching:!1}),this._hand}getTargetRaySpace(){return this._targetRay===null&&(this._targetRay=new ri,this._targetRay.matrixAutoUpdate=!1,this._targetRay.visible=!1,this._targetRay.hasLinearVelocity=!1,this._targetRay.linearVelocity=new L,this._targetRay.hasAngularVelocity=!1,this._targetRay.angularVelocity=new L),this._targetRay}getGripSpace(){return this._grip===null&&(this._grip=new ri,this._grip.matrixAutoUpdate=!1,this._grip.visible=!1,this._grip.hasLinearVelocity=!1,this._grip.linearVelocity=new L,this._grip.hasAngularVelocity=!1,this._grip.angularVelocity=new L),this._grip}dispatchEvent(e){return this._targetRay!==null&&this._targetRay.dispatchEvent(e),this._grip!==null&&this._grip.dispatchEvent(e),this._hand!==null&&this._hand.dispatchEvent(e),this}connect(e){if(e&&e.hand){const t=this._hand;if(t)for(const i of e.hand.values())this._getHandJoint(t,i)}return this.dispatchEvent({type:"connected",data:e}),this}disconnect(e){return this.dispatchEvent({type:"disconnected",data:e}),this._targetRay!==null&&(this._targetRay.visible=!1),this._grip!==null&&(this._grip.visible=!1),this._hand!==null&&(this._hand.visible=!1),this}update(e,t,i){let r=null,s=null,o=null;const a=this._targetRay,l=this._grip,c=this._hand;if(e&&t.session.visibilityState!=="visible-blurred"){if(c&&e.hand){o=!0;for(const y of e.hand.values()){const m=t.getJointPose(y,i),h=this._getHandJoint(c,y);m!==null&&(h.matrix.fromArray(m.transform.matrix),h.matrix.decompose(h.position,h.rotation,h.scale),h.matrixWorldNeedsUpdate=!0,h.jointRadius=m.radius),h.visible=m!==null}const u=c.joints["index-finger-tip"],d=c.joints["thumb-tip"],f=u.position.distanceTo(d.position),p=.02,g=.005;c.inputState.pinching&&f>p+g?(c.inputState.pinching=!1,this.dispatchEvent({type:"pinchend",handedness:e.handedness,target:this})):!c.inputState.pinching&&f<=p-g&&(c.inputState.pinching=!0,this.dispatchEvent({type:"pinchstart",handedness:e.handedness,target:this}))}else l!==null&&e.gripSpace&&(s=t.getPose(e.gripSpace,i),s!==null&&(l.matrix.fromArray(s.transform.matrix),l.matrix.decompose(l.position,l.rotation,l.scale),l.matrixWorldNeedsUpdate=!0,s.linearVelocity?(l.hasLinearVelocity=!0,l.linearVelocity.copy(s.linearVelocity)):l.hasLinearVelocity=!1,s.angularVelocity?(l.hasAngularVelocity=!0,l.angularVelocity.copy(s.angularVelocity)):l.hasAngularVelocity=!1));a!==null&&(r=t.getPose(e.targetRaySpace,i),r===null&&s!==null&&(r=s),r!==null&&(a.matrix.fromArray(r.transform.matrix),a.matrix.decompose(a.position,a.rotation,a.scale),a.matrixWorldNeedsUpdate=!0,r.linearVelocity?(a.hasLinearVelocity=!0,a.linearVelocity.copy(r.linearVelocity)):a.hasLinearVelocity=!1,r.angularVelocity?(a.hasAngularVelocity=!0,a.angularVelocity.copy(r.angularVelocity)):a.hasAngularVelocity=!1,this.dispatchEvent(hR)))}return a!==null&&(a.visible=r!==null),l!==null&&(l.visible=s!==null),c!==null&&(c.visible=o!==null),this}_getHandJoint(e,t){if(e.joints[t.jointName]===void 0){const i=new ri;i.matrixAutoUpdate=!1,i.visible=!1,e.joints[t.jointName]=i,e.add(i)}return e.joints[t.jointName]}}const pR=`
void main() {

	gl_Position = vec4( position, 1.0 );

}`,mR=`
uniform sampler2DArray depthColor;
uniform float depthWidth;
uniform float depthHeight;

void main() {

	vec2 coord = vec2( gl_FragCoord.x / depthWidth, gl_FragCoord.y / depthHeight );

	if ( coord.x >= 1.0 ) {

		gl_FragDepth = texture( depthColor, vec3( coord.x - 1.0, coord.y, 1 ) ).r;

	} else {

		gl_FragDepth = texture( depthColor, vec3( coord.x, coord.y, 0 ) ).r;

	}

}`;class gR{constructor(){this.texture=null,this.mesh=null,this.depthNear=0,this.depthFar=0}init(e,t,i){if(this.texture===null){const r=new Pt,s=e.properties.get(r);s.__webglTexture=t.texture,(t.depthNear!=i.depthNear||t.depthFar!=i.depthFar)&&(this.depthNear=t.depthNear,this.depthFar=t.depthFar),this.texture=r}}getMesh(e){if(this.texture!==null&&this.mesh===null){const t=e.cameras[0].viewport,i=new Di({vertexShader:pR,fragmentShader:mR,uniforms:{depthColor:{value:this.texture},depthWidth:{value:t.z},depthHeight:{value:t.w}}});this.mesh=new tn(new Vc(20,20),i)}return this.mesh}reset(){this.texture=null,this.mesh=null}getDepthTexture(){return this.texture}}class _R extends so{constructor(e,t){super();const i=this;let r=null,s=1,o=null,a="local-floor",l=1,c=null,u=null,d=null,f=null,p=null,g=null;const y=new gR,m=t.getContextAttributes();let h=null,_=null;const v=[],x=[],R=new we;let A=null;const w=new Jt;w.layers.enable(1),w.viewport=new st;const b=new Jt;b.layers.enable(2),b.viewport=new st;const T=[w,b],M=new fR;M.layers.enable(1),M.layers.enable(2);let I=null,V=null;this.cameraAutoUpdate=!0,this.enabled=!1,this.isPresenting=!1,this.getController=function(G){let ne=v[G];return ne===void 0&&(ne=new ju,v[G]=ne),ne.getTargetRaySpace()},this.getControllerGrip=function(G){let ne=v[G];return ne===void 0&&(ne=new ju,v[G]=ne),ne.getGripSpace()},this.getHand=function(G){let ne=v[G];return ne===void 0&&(ne=new ju,v[G]=ne),ne.getHandSpace()};function B(G){const ne=x.indexOf(G.inputSource);if(ne===-1)return;const he=v[ne];he!==void 0&&(he.update(G.inputSource,G.frame,c||o),he.dispatchEvent({type:G.type,data:G.inputSource}))}function j(){r.removeEventListener("select",B),r.removeEventListener("selectstart",B),r.removeEventListener("selectend",B),r.removeEventListener("squeeze",B),r.removeEventListener("squeezestart",B),r.removeEventListener("squeezeend",B),r.removeEventListener("end",j),r.removeEventListener("inputsourceschange",Y);for(let G=0;G<v.length;G++){const ne=x[G];ne!==null&&(x[G]=null,v[G].disconnect(ne))}I=null,V=null,y.reset(),e.setRenderTarget(h),p=null,f=null,d=null,r=null,_=null,je.stop(),i.isPresenting=!1,e.setPixelRatio(A),e.setSize(R.width,R.height,!1),i.dispatchEvent({type:"sessionend"})}this.setFramebufferScaleFactor=function(G){s=G,i.isPresenting===!0&&console.warn("THREE.WebXRManager: Cannot change framebuffer scale while presenting.")},this.setReferenceSpaceType=function(G){a=G,i.isPresenting===!0&&console.warn("THREE.WebXRManager: Cannot change reference space type while presenting.")},this.getReferenceSpace=function(){return c||o},this.setReferenceSpace=function(G){c=G},this.getBaseLayer=function(){return f!==null?f:p},this.getBinding=function(){return d},this.getFrame=function(){return g},this.getSession=function(){return r},this.setSession=async function(G){if(r=G,r!==null){if(h=e.getRenderTarget(),r.addEventListener("select",B),r.addEventListener("selectstart",B),r.addEventListener("selectend",B),r.addEventListener("squeeze",B),r.addEventListener("squeezestart",B),r.addEventListener("squeezeend",B),r.addEventListener("end",j),r.addEventListener("inputsourceschange",Y),m.xrCompatible!==!0&&await t.makeXRCompatible(),A=e.getPixelRatio(),e.getSize(R),r.renderState.layers===void 0){const ne={antialias:m.antialias,alpha:!0,depth:m.depth,stencil:m.stencil,framebufferScaleFactor:s};p=new XRWebGLLayer(r,t,ne),r.updateRenderState({baseLayer:p}),e.setPixelRatio(1),e.setSize(p.framebufferWidth,p.framebufferHeight,!1),_=new Vr(p.framebufferWidth,p.framebufferHeight,{format:Pn,type:Ni,colorSpace:e.outputColorSpace,stencilBuffer:m.stencil})}else{let ne=null,he=null,de=null;m.depth&&(de=m.stencil?t.DEPTH24_STENCIL8:t.DEPTH_COMPONENT24,ne=m.stencil?$s:Us,he=m.stencil?qs:Hr);const Ce={colorFormat:t.RGBA8,depthFormat:de,scaleFactor:s};d=new XRWebGLBinding(r,t),f=d.createProjectionLayer(Ce),r.updateRenderState({layers:[f]}),e.setPixelRatio(1),e.setSize(f.textureWidth,f.textureHeight,!1),_=new Vr(f.textureWidth,f.textureHeight,{format:Pn,type:Ni,depthTexture:new Gv(f.textureWidth,f.textureHeight,he,void 0,void 0,void 0,void 0,void 0,void 0,ne),stencilBuffer:m.stencil,colorSpace:e.outputColorSpace,samples:m.antialias?4:0,resolveDepthBuffer:f.ignoreDepthValues===!1})}_.isXRRenderTarget=!0,this.setFoveation(l),c=null,o=await r.requestReferenceSpace(a),je.setContext(r),je.start(),i.isPresenting=!0,i.dispatchEvent({type:"sessionstart"})}},this.getEnvironmentBlendMode=function(){if(r!==null)return r.environmentBlendMode},this.getDepthTexture=function(){return y.getDepthTexture()};function Y(G){for(let ne=0;ne<G.removed.length;ne++){const he=G.removed[ne],de=x.indexOf(he);de>=0&&(x[de]=null,v[de].disconnect(he))}for(let ne=0;ne<G.added.length;ne++){const he=G.added[ne];let de=x.indexOf(he);if(de===-1){for(let Fe=0;Fe<v.length;Fe++)if(Fe>=x.length){x.push(he),de=Fe;break}else if(x[Fe]===null){x[Fe]=he,de=Fe;break}if(de===-1)break}const Ce=v[de];Ce&&Ce.connect(he)}}const W=new L,$=new L;function N(G,ne,he){W.setFromMatrixPosition(ne.matrixWorld),$.setFromMatrixPosition(he.matrixWorld);const de=W.distanceTo($),Ce=ne.projectionMatrix.elements,Fe=he.projectionMatrix.elements,We=Ce[14]/(Ce[10]-1),gt=Ce[14]/(Ce[10]+1),P=(Ce[9]+1)/Ce[5],St=(Ce[9]-1)/Ce[5],et=(Ce[8]-1)/Ce[0],rt=(Fe[8]+1)/Fe[0],Ee=We*et,Mt=We*rt,Ie=de/(-et+rt),Oe=Ie*-et;if(ne.matrixWorld.decompose(G.position,G.quaternion,G.scale),G.translateX(Oe),G.translateZ(Ie),G.matrixWorld.compose(G.position,G.quaternion,G.scale),G.matrixWorldInverse.copy(G.matrixWorld).invert(),Ce[10]===-1)G.projectionMatrix.copy(ne.projectionMatrix),G.projectionMatrixInverse.copy(ne.projectionMatrixInverse);else{const C=We+Ie,S=gt+Ie,z=Ee-Oe,Z=Mt+(de-Oe),ee=P*gt/S*C,J=St*gt/S*C;G.projectionMatrix.makePerspective(z,Z,ee,J,C,S),G.projectionMatrixInverse.copy(G.projectionMatrix).invert()}}function K(G,ne){ne===null?G.matrixWorld.copy(G.matrix):G.matrixWorld.multiplyMatrices(ne.matrixWorld,G.matrix),G.matrixWorldInverse.copy(G.matrixWorld).invert()}this.updateCamera=function(G){if(r===null)return;let ne=G.near,he=G.far;y.texture!==null&&(y.depthNear>0&&(ne=y.depthNear),y.depthFar>0&&(he=y.depthFar)),M.near=b.near=w.near=ne,M.far=b.far=w.far=he,(I!==M.near||V!==M.far)&&(r.updateRenderState({depthNear:M.near,depthFar:M.far}),I=M.near,V=M.far);const de=G.parent,Ce=M.cameras;K(M,de);for(let Fe=0;Fe<Ce.length;Fe++)K(Ce[Fe],de);Ce.length===2?N(M,w,b):M.projectionMatrix.copy(w.projectionMatrix),Q(G,M,de)};function Q(G,ne,he){he===null?G.matrix.copy(ne.matrixWorld):(G.matrix.copy(he.matrixWorld),G.matrix.invert(),G.matrix.multiply(ne.matrixWorld)),G.matrix.decompose(G.position,G.quaternion,G.scale),G.updateMatrixWorld(!0),G.projectionMatrix.copy(ne.projectionMatrix),G.projectionMatrixInverse.copy(ne.projectionMatrixInverse),G.isPerspectiveCamera&&(G.fov=Zs*2*Math.atan(1/G.projectionMatrix.elements[5]),G.zoom=1)}this.getCamera=function(){return M},this.getFoveation=function(){if(!(f===null&&p===null))return l},this.setFoveation=function(G){l=G,f!==null&&(f.fixedFoveation=G),p!==null&&p.fixedFoveation!==void 0&&(p.fixedFoveation=G)},this.hasDepthSensing=function(){return y.texture!==null},this.getDepthSensingMesh=function(){return y.getMesh(M)};let oe=null;function Me(G,ne){if(u=ne.getViewerPose(c||o),g=ne,u!==null){const he=u.views;p!==null&&(e.setRenderTargetFramebuffer(_,p.framebuffer),e.setRenderTarget(_));let de=!1;he.length!==M.cameras.length&&(M.cameras.length=0,de=!0);for(let Fe=0;Fe<he.length;Fe++){const We=he[Fe];let gt=null;if(p!==null)gt=p.getViewport(We);else{const St=d.getViewSubImage(f,We);gt=St.viewport,Fe===0&&(e.setRenderTargetTextures(_,St.colorTexture,f.ignoreDepthValues?void 0:St.depthStencilTexture),e.setRenderTarget(_))}let P=T[Fe];P===void 0&&(P=new Jt,P.layers.enable(Fe),P.viewport=new st,T[Fe]=P),P.matrix.fromArray(We.transform.matrix),P.matrix.decompose(P.position,P.quaternion,P.scale),P.projectionMatrix.fromArray(We.projectionMatrix),P.projectionMatrixInverse.copy(P.projectionMatrix).invert(),P.viewport.set(gt.x,gt.y,gt.width,gt.height),Fe===0&&(M.matrix.copy(P.matrix),M.matrix.decompose(M.position,M.quaternion,M.scale)),de===!0&&M.cameras.push(P)}const Ce=r.enabledFeatures;if(Ce&&Ce.includes("depth-sensing")){const Fe=d.getDepthInformation(he[0]);Fe&&Fe.isValid&&Fe.texture&&y.init(e,Fe,r.renderState)}}for(let he=0;he<v.length;he++){const de=x[he],Ce=v[he];de!==null&&Ce!==void 0&&Ce.update(de,ne,c||o)}oe&&oe(G,ne),ne.detectedPlanes&&i.dispatchEvent({type:"planesdetected",data:ne}),g=null}const je=new Vv;je.setAnimationLoop(Me),this.setAnimationLoop=function(G){oe=G},this.dispose=function(){}}}const Er=new ai,vR=new Ue;function yR(n,e){function t(m,h){m.matrixAutoUpdate===!0&&m.updateMatrix(),h.value.copy(m.matrix)}function i(m,h){h.color.getRGB(m.fogColor.value,Bv(n)),h.isFog?(m.fogNear.value=h.near,m.fogFar.value=h.far):h.isFogExp2&&(m.fogDensity.value=h.density)}function r(m,h,_,v,x){h.isMeshBasicMaterial||h.isMeshLambertMaterial?s(m,h):h.isMeshToonMaterial?(s(m,h),d(m,h)):h.isMeshPhongMaterial?(s(m,h),u(m,h)):h.isMeshStandardMaterial?(s(m,h),f(m,h),h.isMeshPhysicalMaterial&&p(m,h,x)):h.isMeshMatcapMaterial?(s(m,h),g(m,h)):h.isMeshDepthMaterial?s(m,h):h.isMeshDistanceMaterial?(s(m,h),y(m,h)):h.isMeshNormalMaterial?s(m,h):h.isLineBasicMaterial?(o(m,h),h.isLineDashedMaterial&&a(m,h)):h.isPointsMaterial?l(m,h,_,v):h.isSpriteMaterial?c(m,h):h.isShadowMaterial?(m.color.value.copy(h.color),m.opacity.value=h.opacity):h.isShaderMaterial&&(h.uniformsNeedUpdate=!1)}function s(m,h){m.opacity.value=h.opacity,h.color&&m.diffuse.value.copy(h.color),h.emissive&&m.emissive.value.copy(h.emissive).multiplyScalar(h.emissiveIntensity),h.map&&(m.map.value=h.map,t(h.map,m.mapTransform)),h.alphaMap&&(m.alphaMap.value=h.alphaMap,t(h.alphaMap,m.alphaMapTransform)),h.bumpMap&&(m.bumpMap.value=h.bumpMap,t(h.bumpMap,m.bumpMapTransform),m.bumpScale.value=h.bumpScale,h.side===fn&&(m.bumpScale.value*=-1)),h.normalMap&&(m.normalMap.value=h.normalMap,t(h.normalMap,m.normalMapTransform),m.normalScale.value.copy(h.normalScale),h.side===fn&&m.normalScale.value.negate()),h.displacementMap&&(m.displacementMap.value=h.displacementMap,t(h.displacementMap,m.displacementMapTransform),m.displacementScale.value=h.displacementScale,m.displacementBias.value=h.displacementBias),h.emissiveMap&&(m.emissiveMap.value=h.emissiveMap,t(h.emissiveMap,m.emissiveMapTransform)),h.specularMap&&(m.specularMap.value=h.specularMap,t(h.specularMap,m.specularMapTransform)),h.alphaTest>0&&(m.alphaTest.value=h.alphaTest);const _=e.get(h),v=_.envMap,x=_.envMapRotation;v&&(m.envMap.value=v,Er.copy(x),Er.x*=-1,Er.y*=-1,Er.z*=-1,v.isCubeTexture&&v.isRenderTargetTexture===!1&&(Er.y*=-1,Er.z*=-1),m.envMapRotation.value.setFromMatrix4(vR.makeRotationFromEuler(Er)),m.flipEnvMap.value=v.isCubeTexture&&v.isRenderTargetTexture===!1?-1:1,m.reflectivity.value=h.reflectivity,m.ior.value=h.ior,m.refractionRatio.value=h.refractionRatio),h.lightMap&&(m.lightMap.value=h.lightMap,m.lightMapIntensity.value=h.lightMapIntensity,t(h.lightMap,m.lightMapTransform)),h.aoMap&&(m.aoMap.value=h.aoMap,m.aoMapIntensity.value=h.aoMapIntensity,t(h.aoMap,m.aoMapTransform))}function o(m,h){m.diffuse.value.copy(h.color),m.opacity.value=h.opacity,h.map&&(m.map.value=h.map,t(h.map,m.mapTransform))}function a(m,h){m.dashSize.value=h.dashSize,m.totalSize.value=h.dashSize+h.gapSize,m.scale.value=h.scale}function l(m,h,_,v){m.diffuse.value.copy(h.color),m.opacity.value=h.opacity,m.size.value=h.size*_,m.scale.value=v*.5,h.map&&(m.map.value=h.map,t(h.map,m.uvTransform)),h.alphaMap&&(m.alphaMap.value=h.alphaMap,t(h.alphaMap,m.alphaMapTransform)),h.alphaTest>0&&(m.alphaTest.value=h.alphaTest)}function c(m,h){m.diffuse.value.copy(h.color),m.opacity.value=h.opacity,m.rotation.value=h.rotation,h.map&&(m.map.value=h.map,t(h.map,m.mapTransform)),h.alphaMap&&(m.alphaMap.value=h.alphaMap,t(h.alphaMap,m.alphaMapTransform)),h.alphaTest>0&&(m.alphaTest.value=h.alphaTest)}function u(m,h){m.specular.value.copy(h.specular),m.shininess.value=Math.max(h.shininess,1e-4)}function d(m,h){h.gradientMap&&(m.gradientMap.value=h.gradientMap)}function f(m,h){m.metalness.value=h.metalness,h.metalnessMap&&(m.metalnessMap.value=h.metalnessMap,t(h.metalnessMap,m.metalnessMapTransform)),m.roughness.value=h.roughness,h.roughnessMap&&(m.roughnessMap.value=h.roughnessMap,t(h.roughnessMap,m.roughnessMapTransform)),h.envMap&&(m.envMapIntensity.value=h.envMapIntensity)}function p(m,h,_){m.ior.value=h.ior,h.sheen>0&&(m.sheenColor.value.copy(h.sheenColor).multiplyScalar(h.sheen),m.sheenRoughness.value=h.sheenRoughness,h.sheenColorMap&&(m.sheenColorMap.value=h.sheenColorMap,t(h.sheenColorMap,m.sheenColorMapTransform)),h.sheenRoughnessMap&&(m.sheenRoughnessMap.value=h.sheenRoughnessMap,t(h.sheenRoughnessMap,m.sheenRoughnessMapTransform))),h.clearcoat>0&&(m.clearcoat.value=h.clearcoat,m.clearcoatRoughness.value=h.clearcoatRoughness,h.clearcoatMap&&(m.clearcoatMap.value=h.clearcoatMap,t(h.clearcoatMap,m.clearcoatMapTransform)),h.clearcoatRoughnessMap&&(m.clearcoatRoughnessMap.value=h.clearcoatRoughnessMap,t(h.clearcoatRoughnessMap,m.clearcoatRoughnessMapTransform)),h.clearcoatNormalMap&&(m.clearcoatNormalMap.value=h.clearcoatNormalMap,t(h.clearcoatNormalMap,m.clearcoatNormalMapTransform),m.clearcoatNormalScale.value.copy(h.clearcoatNormalScale),h.side===fn&&m.clearcoatNormalScale.value.negate())),h.dispersion>0&&(m.dispersion.value=h.dispersion),h.iridescence>0&&(m.iridescence.value=h.iridescence,m.iridescenceIOR.value=h.iridescenceIOR,m.iridescenceThicknessMinimum.value=h.iridescenceThicknessRange[0],m.iridescenceThicknessMaximum.value=h.iridescenceThicknessRange[1],h.iridescenceMap&&(m.iridescenceMap.value=h.iridescenceMap,t(h.iridescenceMap,m.iridescenceMapTransform)),h.iridescenceThicknessMap&&(m.iridescenceThicknessMap.value=h.iridescenceThicknessMap,t(h.iridescenceThicknessMap,m.iridescenceThicknessMapTransform))),h.transmission>0&&(m.transmission.value=h.transmission,m.transmissionSamplerMap.value=_.texture,m.transmissionSamplerSize.value.set(_.width,_.height),h.transmissionMap&&(m.transmissionMap.value=h.transmissionMap,t(h.transmissionMap,m.transmissionMapTransform)),m.thickness.value=h.thickness,h.thicknessMap&&(m.thicknessMap.value=h.thicknessMap,t(h.thicknessMap,m.thicknessMapTransform)),m.attenuationDistance.value=h.attenuationDistance,m.attenuationColor.value.copy(h.attenuationColor)),h.anisotropy>0&&(m.anisotropyVector.value.set(h.anisotropy*Math.cos(h.anisotropyRotation),h.anisotropy*Math.sin(h.anisotropyRotation)),h.anisotropyMap&&(m.anisotropyMap.value=h.anisotropyMap,t(h.anisotropyMap,m.anisotropyMapTransform))),m.specularIntensity.value=h.specularIntensity,m.specularColor.value.copy(h.specularColor),h.specularColorMap&&(m.specularColorMap.value=h.specularColorMap,t(h.specularColorMap,m.specularColorMapTransform)),h.specularIntensityMap&&(m.specularIntensityMap.value=h.specularIntensityMap,t(h.specularIntensityMap,m.specularIntensityMapTransform))}function g(m,h){h.matcap&&(m.matcap.value=h.matcap)}function y(m,h){const _=e.get(h).light;m.referencePosition.value.setFromMatrixPosition(_.matrixWorld),m.nearDistance.value=_.shadow.camera.near,m.farDistance.value=_.shadow.camera.far}return{refreshFogUniforms:i,refreshMaterialUniforms:r}}function xR(n,e,t,i){let r={},s={},o=[];const a=n.getParameter(n.MAX_UNIFORM_BUFFER_BINDINGS);function l(_,v){const x=v.program;i.uniformBlockBinding(_,x)}function c(_,v){let x=r[_.id];x===void 0&&(g(_),x=u(_),r[_.id]=x,_.addEventListener("dispose",m));const R=v.program;i.updateUBOMapping(_,R);const A=e.render.frame;s[_.id]!==A&&(f(_),s[_.id]=A)}function u(_){const v=d();_.__bindingPointIndex=v;const x=n.createBuffer(),R=_.__size,A=_.usage;return n.bindBuffer(n.UNIFORM_BUFFER,x),n.bufferData(n.UNIFORM_BUFFER,R,A),n.bindBuffer(n.UNIFORM_BUFFER,null),n.bindBufferBase(n.UNIFORM_BUFFER,v,x),x}function d(){for(let _=0;_<a;_++)if(o.indexOf(_)===-1)return o.push(_),_;return console.error("THREE.WebGLRenderer: Maximum number of simultaneously usable uniforms groups reached."),0}function f(_){const v=r[_.id],x=_.uniforms,R=_.__cache;n.bindBuffer(n.UNIFORM_BUFFER,v);for(let A=0,w=x.length;A<w;A++){const b=Array.isArray(x[A])?x[A]:[x[A]];for(let T=0,M=b.length;T<M;T++){const I=b[T];if(p(I,A,T,R)===!0){const V=I.__offset,B=Array.isArray(I.value)?I.value:[I.value];let j=0;for(let Y=0;Y<B.length;Y++){const W=B[Y],$=y(W);typeof W=="number"||typeof W=="boolean"?(I.__data[0]=W,n.bufferSubData(n.UNIFORM_BUFFER,V+j,I.__data)):W.isMatrix3?(I.__data[0]=W.elements[0],I.__data[1]=W.elements[1],I.__data[2]=W.elements[2],I.__data[3]=0,I.__data[4]=W.elements[3],I.__data[5]=W.elements[4],I.__data[6]=W.elements[5],I.__data[7]=0,I.__data[8]=W.elements[6],I.__data[9]=W.elements[7],I.__data[10]=W.elements[8],I.__data[11]=0):(W.toArray(I.__data,j),j+=$.storage/Float32Array.BYTES_PER_ELEMENT)}n.bufferSubData(n.UNIFORM_BUFFER,V,I.__data)}}}n.bindBuffer(n.UNIFORM_BUFFER,null)}function p(_,v,x,R){const A=_.value,w=v+"_"+x;if(R[w]===void 0)return typeof A=="number"||typeof A=="boolean"?R[w]=A:R[w]=A.clone(),!0;{const b=R[w];if(typeof A=="number"||typeof A=="boolean"){if(b!==A)return R[w]=A,!0}else if(b.equals(A)===!1)return b.copy(A),!0}return!1}function g(_){const v=_.uniforms;let x=0;const R=16;for(let w=0,b=v.length;w<b;w++){const T=Array.isArray(v[w])?v[w]:[v[w]];for(let M=0,I=T.length;M<I;M++){const V=T[M],B=Array.isArray(V.value)?V.value:[V.value];for(let j=0,Y=B.length;j<Y;j++){const W=B[j],$=y(W),N=x%R,K=N%$.boundary,Q=N+K;x+=K,Q!==0&&R-Q<$.storage&&(x+=R-Q),V.__data=new Float32Array($.storage/Float32Array.BYTES_PER_ELEMENT),V.__offset=x,x+=$.storage}}}const A=x%R;return A>0&&(x+=R-A),_.__size=x,_.__cache={},this}function y(_){const v={boundary:0,storage:0};return typeof _=="number"||typeof _=="boolean"?(v.boundary=4,v.storage=4):_.isVector2?(v.boundary=8,v.storage=8):_.isVector3||_.isColor?(v.boundary=16,v.storage=12):_.isVector4?(v.boundary=16,v.storage=16):_.isMatrix3?(v.boundary=48,v.storage=48):_.isMatrix4?(v.boundary=64,v.storage=64):_.isTexture?console.warn("THREE.WebGLRenderer: Texture samplers can not be part of an uniforms group."):console.warn("THREE.WebGLRenderer: Unsupported uniform value type.",_),v}function m(_){const v=_.target;v.removeEventListener("dispose",m);const x=o.indexOf(v.__bindingPointIndex);o.splice(x,1),n.deleteBuffer(r[v.id]),delete r[v.id],delete s[v.id]}function h(){for(const _ in r)n.deleteBuffer(r[_]);o=[],r={},s={}}return{bind:l,update:c,dispose:h}}class SR{constructor(e={}){const{canvas:t=dE(),context:i=null,depth:r=!0,stencil:s=!1,alpha:o=!1,antialias:a=!1,premultipliedAlpha:l=!0,preserveDrawingBuffer:c=!1,powerPreference:u="default",failIfMajorPerformanceCaveat:d=!1}=e;this.isWebGLRenderer=!0;let f;if(i!==null){if(typeof WebGLRenderingContext<"u"&&i instanceof WebGLRenderingContext)throw new Error("THREE.WebGLRenderer: WebGL 1 is not supported since r163.");f=i.getContextAttributes().alpha}else f=o;const p=new Uint32Array(4),g=new Int32Array(4);let y=null,m=null;const h=[],_=[];this.domElement=t,this.debug={checkShaderErrors:!0,onShaderError:null},this.autoClear=!0,this.autoClearColor=!0,this.autoClearDepth=!0,this.autoClearStencil=!0,this.sortObjects=!0,this.clippingPlanes=[],this.localClippingEnabled=!1,this._outputColorSpace=Qt,this.toneMapping=ur,this.toneMappingExposure=1;const v=this;let x=!1,R=0,A=0,w=null,b=-1,T=null;const M=new st,I=new st;let V=null;const B=new Pe(0);let j=0,Y=t.width,W=t.height,$=1,N=null,K=null;const Q=new st(0,0,Y,W),oe=new st(0,0,Y,W);let Me=!1;const je=new Kh;let G=!1,ne=!1;const he=new Ue,de=new L,Ce=new st,Fe={background:null,fog:null,environment:null,overrideMaterial:null,isScene:!0};let We=!1;function gt(){return w===null?$:1}let P=i;function St(E,U){return t.getContext(E,U)}try{const E={alpha:!0,depth:r,stencil:s,antialias:a,premultipliedAlpha:l,preserveDrawingBuffer:c,powerPreference:u,failIfMajorPerformanceCaveat:d};if("setAttribute"in t&&t.setAttribute("data-engine",`three.js r${Oh}`),t.addEventListener("webglcontextlost",X,!1),t.addEventListener("webglcontextrestored",q,!1),t.addEventListener("webglcontextcreationerror",ae,!1),P===null){const U="webgl2";if(P=St(U,E),P===null)throw St(U)?new Error("Error creating WebGL context with your selected attributes."):new Error("Error creating WebGL context.")}}catch(E){throw console.error("THREE.WebGLRenderer: "+E.message),E}let et,rt,Ee,Mt,Ie,Oe,C,S,z,Z,ee,J,Te,ce,ge,ke,ie,me,Xe,Le,_e,De,He,ft;function D(){et=new RA(P),et.init(),De=new dR(P,et),rt=new SA(P,et,e,De),Ee=new lR(P),Mt=new PA(P),Ie=new K1,Oe=new uR(P,et,Ee,Ie,rt,De,Mt),C=new EA(v),S=new AA(v),z=new OE(P),He=new yA(P,z),Z=new CA(P,z,Mt,He),ee=new IA(P,Z,z,Mt),Xe=new LA(P,rt,Oe),ke=new MA(Ie),J=new Y1(v,C,S,et,rt,He,ke),Te=new yR(v,Ie),ce=new $1,ge=new nR(et),me=new vA(v,C,S,Ee,ee,f,l),ie=new aR(v,ee,rt),ft=new xR(P,Mt,rt,Ee),Le=new xA(P,et,Mt),_e=new bA(P,et,Mt),Mt.programs=J.programs,v.capabilities=rt,v.extensions=et,v.properties=Ie,v.renderLists=ce,v.shadowMap=ie,v.state=Ee,v.info=Mt}D();const re=new _R(v,P);this.xr=re,this.getContext=function(){return P},this.getContextAttributes=function(){return P.getContextAttributes()},this.forceContextLoss=function(){const E=et.get("WEBGL_lose_context");E&&E.loseContext()},this.forceContextRestore=function(){const E=et.get("WEBGL_lose_context");E&&E.restoreContext()},this.getPixelRatio=function(){return $},this.setPixelRatio=function(E){E!==void 0&&($=E,this.setSize(Y,W,!1))},this.getSize=function(E){return E.set(Y,W)},this.setSize=function(E,U,O=!0){if(re.isPresenting){console.warn("THREE.WebGLRenderer: Can't change size while VR device is presenting.");return}Y=E,W=U,t.width=Math.floor(E*$),t.height=Math.floor(U*$),O===!0&&(t.style.width=E+"px",t.style.height=U+"px"),this.setViewport(0,0,E,U)},this.getDrawingBufferSize=function(E){return E.set(Y*$,W*$).floor()},this.setDrawingBufferSize=function(E,U,O){Y=E,W=U,$=O,t.width=Math.floor(E*O),t.height=Math.floor(U*O),this.setViewport(0,0,E,U)},this.getCurrentViewport=function(E){return E.copy(M)},this.getViewport=function(E){return E.copy(Q)},this.setViewport=function(E,U,O,k){E.isVector4?Q.set(E.x,E.y,E.z,E.w):Q.set(E,U,O,k),Ee.viewport(M.copy(Q).multiplyScalar($).round())},this.getScissor=function(E){return E.copy(oe)},this.setScissor=function(E,U,O,k){E.isVector4?oe.set(E.x,E.y,E.z,E.w):oe.set(E,U,O,k),Ee.scissor(I.copy(oe).multiplyScalar($).round())},this.getScissorTest=function(){return Me},this.setScissorTest=function(E){Ee.setScissorTest(Me=E)},this.setOpaqueSort=function(E){N=E},this.setTransparentSort=function(E){K=E},this.getClearColor=function(E){return E.copy(me.getClearColor())},this.setClearColor=function(){me.setClearColor.apply(me,arguments)},this.getClearAlpha=function(){return me.getClearAlpha()},this.setClearAlpha=function(){me.setClearAlpha.apply(me,arguments)},this.clear=function(E=!0,U=!0,O=!0){let k=0;if(E){let F=!1;if(w!==null){const se=w.texture.format;F=se===Wh||se===Gh||se===Vh}if(F){const se=w.texture.type,fe=se===Ni||se===Hr||se===ha||se===qs||se===Bh||se===zh,ve=me.getClearColor(),ye=me.getClearAlpha(),Re=ve.r,be=ve.g,xe=ve.b;fe?(p[0]=Re,p[1]=be,p[2]=xe,p[3]=ye,P.clearBufferuiv(P.COLOR,0,p)):(g[0]=Re,g[1]=be,g[2]=xe,g[3]=ye,P.clearBufferiv(P.COLOR,0,g))}else k|=P.COLOR_BUFFER_BIT}U&&(k|=P.DEPTH_BUFFER_BIT),O&&(k|=P.STENCIL_BUFFER_BIT,this.state.buffers.stencil.setMask(4294967295)),P.clear(k)},this.clearColor=function(){this.clear(!0,!1,!1)},this.clearDepth=function(){this.clear(!1,!0,!1)},this.clearStencil=function(){this.clear(!1,!1,!0)},this.dispose=function(){t.removeEventListener("webglcontextlost",X,!1),t.removeEventListener("webglcontextrestored",q,!1),t.removeEventListener("webglcontextcreationerror",ae,!1),ce.dispose(),ge.dispose(),Ie.dispose(),C.dispose(),S.dispose(),ee.dispose(),He.dispose(),ft.dispose(),J.dispose(),re.dispose(),re.removeEventListener("sessionstart",Zn),re.removeEventListener("sessionend",op),_r.stop()};function X(E){E.preventDefault(),console.log("THREE.WebGLRenderer: Context Lost."),x=!0}function q(){console.log("THREE.WebGLRenderer: Context Restored."),x=!1;const E=Mt.autoReset,U=ie.enabled,O=ie.autoUpdate,k=ie.needsUpdate,F=ie.type;D(),Mt.autoReset=E,ie.enabled=U,ie.autoUpdate=O,ie.needsUpdate=k,ie.type=F}function ae(E){console.error("THREE.WebGLRenderer: A WebGL context could not be created. Reason: ",E.statusMessage)}function Ae(E){const U=E.target;U.removeEventListener("dispose",Ae),Ye(U)}function Ye(E){Et(E),Ie.remove(E)}function Et(E){const U=Ie.get(E).programs;U!==void 0&&(U.forEach(function(O){J.releaseProgram(O)}),E.isShaderMaterial&&J.releaseShaderCache(E))}this.renderBufferDirect=function(E,U,O,k,F,se){U===null&&(U=Fe);const fe=F.isMesh&&F.matrixWorld.determinant()<0,ve=oy(E,U,O,k,F);Ee.setMaterial(k,fe);let ye=O.index,Re=1;if(k.wireframe===!0){if(ye=Z.getWireframeAttribute(O),ye===void 0)return;Re=2}const be=O.drawRange,xe=O.attributes.position;let Ze=be.start*Re,_t=(be.start+be.count)*Re;se!==null&&(Ze=Math.max(Ze,se.start*Re),_t=Math.min(_t,(se.start+se.count)*Re)),ye!==null?(Ze=Math.max(Ze,0),_t=Math.min(_t,ye.count)):xe!=null&&(Ze=Math.max(Ze,0),_t=Math.min(_t,xe.count));const vt=_t-Ze;if(vt<0||vt===1/0)return;He.setup(F,k,ve,O,ye);let pn,Qe=Le;if(ye!==null&&(pn=z.get(ye),Qe=_e,Qe.setIndex(pn)),F.isMesh)k.wireframe===!0?(Ee.setLineWidth(k.wireframeLinewidth*gt()),Qe.setMode(P.LINES)):Qe.setMode(P.TRIANGLES);else if(F.isLine){let Se=k.linewidth;Se===void 0&&(Se=1),Ee.setLineWidth(Se*gt()),F.isLineSegments?Qe.setMode(P.LINES):F.isLineLoop?Qe.setMode(P.LINE_LOOP):Qe.setMode(P.LINE_STRIP)}else F.isPoints?Qe.setMode(P.POINTS):F.isSprite&&Qe.setMode(P.TRIANGLES);if(F.isBatchedMesh)if(F._multiDrawInstances!==null)Qe.renderMultiDrawInstances(F._multiDrawStarts,F._multiDrawCounts,F._multiDrawCount,F._multiDrawInstances);else if(et.get("WEBGL_multi_draw"))Qe.renderMultiDraw(F._multiDrawStarts,F._multiDrawCounts,F._multiDrawCount);else{const Se=F._multiDrawStarts,kt=F._multiDrawCounts,Je=F._multiDrawCount,Un=ye?z.get(ye).bytesPerElement:1,Xr=Ie.get(k).currentProgram.getUniforms();for(let mn=0;mn<Je;mn++)Xr.setValue(P,"_gl_DrawID",mn),Qe.render(Se[mn]/Un,kt[mn])}else if(F.isInstancedMesh)Qe.renderInstances(Ze,vt,F.count);else if(O.isInstancedBufferGeometry){const Se=O._maxInstanceCount!==void 0?O._maxInstanceCount:1/0,kt=Math.min(O.instanceCount,Se);Qe.renderInstances(Ze,vt,kt)}else Qe.render(Ze,vt)};function Ot(E,U,O){E.transparent===!0&&E.side===ii&&E.forceSinglePass===!1?(E.side=fn,E.needsUpdate=!0,ba(E,U,O),E.side=Ii,E.needsUpdate=!0,ba(E,U,O),E.side=ii):ba(E,U,O)}this.compile=function(E,U,O=null){O===null&&(O=E),m=ge.get(O),m.init(U),_.push(m),O.traverseVisible(function(F){F.isLight&&F.layers.test(U.layers)&&(m.pushLight(F),F.castShadow&&m.pushShadow(F))}),E!==O&&E.traverseVisible(function(F){F.isLight&&F.layers.test(U.layers)&&(m.pushLight(F),F.castShadow&&m.pushShadow(F))}),m.setupLights();const k=new Set;return E.traverse(function(F){const se=F.material;if(se)if(Array.isArray(se))for(let fe=0;fe<se.length;fe++){const ve=se[fe];Ot(ve,O,F),k.add(ve)}else Ot(se,O,F),k.add(se)}),_.pop(),m=null,k},this.compileAsync=function(E,U,O=null){const k=this.compile(E,U,O);return new Promise(F=>{function se(){if(k.forEach(function(fe){Ie.get(fe).currentProgram.isReady()&&k.delete(fe)}),k.size===0){F(E);return}setTimeout(se,10)}et.get("KHR_parallel_shader_compile")!==null?se():setTimeout(se,10)})};let $e=null;function di(E){$e&&$e(E)}function Zn(){_r.stop()}function op(){_r.start()}const _r=new Vv;_r.setAnimationLoop(di),typeof self<"u"&&_r.setContext(self),this.setAnimationLoop=function(E){$e=E,re.setAnimationLoop(E),E===null?_r.stop():_r.start()},re.addEventListener("sessionstart",Zn),re.addEventListener("sessionend",op),this.render=function(E,U){if(U!==void 0&&U.isCamera!==!0){console.error("THREE.WebGLRenderer.render: camera is not an instance of THREE.Camera.");return}if(x===!0)return;if(E.matrixWorldAutoUpdate===!0&&E.updateMatrixWorld(),U.parent===null&&U.matrixWorldAutoUpdate===!0&&U.updateMatrixWorld(),re.enabled===!0&&re.isPresenting===!0&&(re.cameraAutoUpdate===!0&&re.updateCamera(U),U=re.getCamera()),E.isScene===!0&&E.onBeforeRender(v,E,U,w),m=ge.get(E,_.length),m.init(U),_.push(m),he.multiplyMatrices(U.projectionMatrix,U.matrixWorldInverse),je.setFromProjectionMatrix(he),ne=this.localClippingEnabled,G=ke.init(this.clippingPlanes,ne),y=ce.get(E,h.length),y.init(),h.push(y),re.enabled===!0&&re.isPresenting===!0){const se=v.xr.getDepthSensingMesh();se!==null&&Xc(se,U,-1/0,v.sortObjects)}Xc(E,U,0,v.sortObjects),y.finish(),v.sortObjects===!0&&y.sort(N,K),We=re.enabled===!1||re.isPresenting===!1||re.hasDepthSensing()===!1,We&&me.addToRenderList(y,E),this.info.render.frame++,G===!0&&ke.beginShadows();const O=m.state.shadowsArray;ie.render(O,E,U),G===!0&&ke.endShadows(),this.info.autoReset===!0&&this.info.reset();const k=y.opaque,F=y.transmissive;if(m.setupLights(),U.isArrayCamera){const se=U.cameras;if(F.length>0)for(let fe=0,ve=se.length;fe<ve;fe++){const ye=se[fe];lp(k,F,E,ye)}We&&me.render(E);for(let fe=0,ve=se.length;fe<ve;fe++){const ye=se[fe];ap(y,E,ye,ye.viewport)}}else F.length>0&&lp(k,F,E,U),We&&me.render(E),ap(y,E,U);w!==null&&(Oe.updateMultisampleRenderTarget(w),Oe.updateRenderTargetMipmap(w)),E.isScene===!0&&E.onAfterRender(v,E,U),He.resetDefaultState(),b=-1,T=null,_.pop(),_.length>0?(m=_[_.length-1],G===!0&&ke.setGlobalState(v.clippingPlanes,m.state.camera)):m=null,h.pop(),h.length>0?y=h[h.length-1]:y=null};function Xc(E,U,O,k){if(E.visible===!1)return;if(E.layers.test(U.layers)){if(E.isGroup)O=E.renderOrder;else if(E.isLOD)E.autoUpdate===!0&&E.update(U);else if(E.isLight)m.pushLight(E),E.castShadow&&m.pushShadow(E);else if(E.isSprite){if(!E.frustumCulled||je.intersectsSprite(E)){k&&Ce.setFromMatrixPosition(E.matrixWorld).applyMatrix4(he);const fe=ee.update(E),ve=E.material;ve.visible&&y.push(E,fe,ve,O,Ce.z,null)}}else if((E.isMesh||E.isLine||E.isPoints)&&(!E.frustumCulled||je.intersectsObject(E))){const fe=ee.update(E),ve=E.material;if(k&&(E.boundingSphere!==void 0?(E.boundingSphere===null&&E.computeBoundingSphere(),Ce.copy(E.boundingSphere.center)):(fe.boundingSphere===null&&fe.computeBoundingSphere(),Ce.copy(fe.boundingSphere.center)),Ce.applyMatrix4(E.matrixWorld).applyMatrix4(he)),Array.isArray(ve)){const ye=fe.groups;for(let Re=0,be=ye.length;Re<be;Re++){const xe=ye[Re],Ze=ve[xe.materialIndex];Ze&&Ze.visible&&y.push(E,fe,Ze,O,Ce.z,xe)}}else ve.visible&&y.push(E,fe,ve,O,Ce.z,null)}}const se=E.children;for(let fe=0,ve=se.length;fe<ve;fe++)Xc(se[fe],U,O,k)}function ap(E,U,O,k){const F=E.opaque,se=E.transmissive,fe=E.transparent;m.setupLightsView(O),G===!0&&ke.setGlobalState(v.clippingPlanes,O),k&&Ee.viewport(M.copy(k)),F.length>0&&Ca(F,U,O),se.length>0&&Ca(se,U,O),fe.length>0&&Ca(fe,U,O),Ee.buffers.depth.setTest(!0),Ee.buffers.depth.setMask(!0),Ee.buffers.color.setMask(!0),Ee.setPolygonOffset(!1)}function lp(E,U,O,k){if((O.isScene===!0?O.overrideMaterial:null)!==null)return;m.state.transmissionRenderTarget[k.id]===void 0&&(m.state.transmissionRenderTarget[k.id]=new Vr(1,1,{generateMipmaps:!0,type:et.has("EXT_color_buffer_half_float")||et.has("EXT_color_buffer_float")?Ta:Ni,minFilter:Ti,samples:4,stencilBuffer:s,resolveDepthBuffer:!1,resolveStencilBuffer:!1,colorSpace:qe.workingColorSpace}));const se=m.state.transmissionRenderTarget[k.id],fe=k.viewport||M;se.setSize(fe.z,fe.w);const ve=v.getRenderTarget();v.setRenderTarget(se),v.getClearColor(B),j=v.getClearAlpha(),j<1&&v.setClearColor(16777215,.5),v.clear(),We&&me.render(O);const ye=v.toneMapping;v.toneMapping=ur;const Re=k.viewport;if(k.viewport!==void 0&&(k.viewport=void 0),m.setupLightsView(k),G===!0&&ke.setGlobalState(v.clippingPlanes,k),Ca(E,O,k),Oe.updateMultisampleRenderTarget(se),Oe.updateRenderTargetMipmap(se),et.has("WEBGL_multisampled_render_to_texture")===!1){let be=!1;for(let xe=0,Ze=U.length;xe<Ze;xe++){const _t=U[xe],vt=_t.object,pn=_t.geometry,Qe=_t.material,Se=_t.group;if(Qe.side===ii&&vt.layers.test(k.layers)){const kt=Qe.side;Qe.side=fn,Qe.needsUpdate=!0,cp(vt,O,k,pn,Qe,Se),Qe.side=kt,Qe.needsUpdate=!0,be=!0}}be===!0&&(Oe.updateMultisampleRenderTarget(se),Oe.updateRenderTargetMipmap(se))}v.setRenderTarget(ve),v.setClearColor(B,j),Re!==void 0&&(k.viewport=Re),v.toneMapping=ye}function Ca(E,U,O){const k=U.isScene===!0?U.overrideMaterial:null;for(let F=0,se=E.length;F<se;F++){const fe=E[F],ve=fe.object,ye=fe.geometry,Re=k===null?fe.material:k,be=fe.group;ve.layers.test(O.layers)&&cp(ve,U,O,ye,Re,be)}}function cp(E,U,O,k,F,se){E.onBeforeRender(v,U,O,k,F,se),E.modelViewMatrix.multiplyMatrices(O.matrixWorldInverse,E.matrixWorld),E.normalMatrix.getNormalMatrix(E.modelViewMatrix),F.onBeforeRender(v,U,O,k,E,se),F.transparent===!0&&F.side===ii&&F.forceSinglePass===!1?(F.side=fn,F.needsUpdate=!0,v.renderBufferDirect(O,U,k,F,E,se),F.side=Ii,F.needsUpdate=!0,v.renderBufferDirect(O,U,k,F,E,se),F.side=ii):v.renderBufferDirect(O,U,k,F,E,se),E.onAfterRender(v,U,O,k,F,se)}function ba(E,U,O){U.isScene!==!0&&(U=Fe);const k=Ie.get(E),F=m.state.lights,se=m.state.shadowsArray,fe=F.state.version,ve=J.getParameters(E,F.state,se,U,O),ye=J.getProgramCacheKey(ve);let Re=k.programs;k.environment=E.isMeshStandardMaterial?U.environment:null,k.fog=U.fog,k.envMap=(E.isMeshStandardMaterial?S:C).get(E.envMap||k.environment),k.envMapRotation=k.environment!==null&&E.envMap===null?U.environmentRotation:E.envMapRotation,Re===void 0&&(E.addEventListener("dispose",Ae),Re=new Map,k.programs=Re);let be=Re.get(ye);if(be!==void 0){if(k.currentProgram===be&&k.lightsStateVersion===fe)return dp(E,ve),be}else ve.uniforms=J.getUniforms(E),E.onBeforeCompile(ve,v),be=J.acquireProgram(ve,ye),Re.set(ye,be),k.uniforms=ve.uniforms;const xe=k.uniforms;return(!E.isShaderMaterial&&!E.isRawShaderMaterial||E.clipping===!0)&&(xe.clippingPlanes=ke.uniform),dp(E,ve),k.needsLights=ly(E),k.lightsStateVersion=fe,k.needsLights&&(xe.ambientLightColor.value=F.state.ambient,xe.lightProbe.value=F.state.probe,xe.directionalLights.value=F.state.directional,xe.directionalLightShadows.value=F.state.directionalShadow,xe.spotLights.value=F.state.spot,xe.spotLightShadows.value=F.state.spotShadow,xe.rectAreaLights.value=F.state.rectArea,xe.ltc_1.value=F.state.rectAreaLTC1,xe.ltc_2.value=F.state.rectAreaLTC2,xe.pointLights.value=F.state.point,xe.pointLightShadows.value=F.state.pointShadow,xe.hemisphereLights.value=F.state.hemi,xe.directionalShadowMap.value=F.state.directionalShadowMap,xe.directionalShadowMatrix.value=F.state.directionalShadowMatrix,xe.spotShadowMap.value=F.state.spotShadowMap,xe.spotLightMatrix.value=F.state.spotLightMatrix,xe.spotLightMap.value=F.state.spotLightMap,xe.pointShadowMap.value=F.state.pointShadowMap,xe.pointShadowMatrix.value=F.state.pointShadowMatrix),k.currentProgram=be,k.uniformsList=null,be}function up(E){if(E.uniformsList===null){const U=E.currentProgram.getUniforms();E.uniformsList=Wl.seqWithValue(U.seq,E.uniforms)}return E.uniformsList}function dp(E,U){const O=Ie.get(E);O.outputColorSpace=U.outputColorSpace,O.batching=U.batching,O.batchingColor=U.batchingColor,O.instancing=U.instancing,O.instancingColor=U.instancingColor,O.instancingMorph=U.instancingMorph,O.skinning=U.skinning,O.morphTargets=U.morphTargets,O.morphNormals=U.morphNormals,O.morphColors=U.morphColors,O.morphTargetsCount=U.morphTargetsCount,O.numClippingPlanes=U.numClippingPlanes,O.numIntersection=U.numClipIntersection,O.vertexAlphas=U.vertexAlphas,O.vertexTangents=U.vertexTangents,O.toneMapping=U.toneMapping}function oy(E,U,O,k,F){U.isScene!==!0&&(U=Fe),Oe.resetTextureUnits();const se=U.fog,fe=k.isMeshStandardMaterial?U.environment:null,ve=w===null?v.outputColorSpace:w.isXRRenderTarget===!0?w.texture.colorSpace:Vt,ye=(k.isMeshStandardMaterial?S:C).get(k.envMap||fe),Re=k.vertexColors===!0&&!!O.attributes.color&&O.attributes.color.itemSize===4,be=!!O.attributes.tangent&&(!!k.normalMap||k.anisotropy>0),xe=!!O.morphAttributes.position,Ze=!!O.morphAttributes.normal,_t=!!O.morphAttributes.color;let vt=ur;k.toneMapped&&(w===null||w.isXRRenderTarget===!0)&&(vt=v.toneMapping);const pn=O.morphAttributes.position||O.morphAttributes.normal||O.morphAttributes.color,Qe=pn!==void 0?pn.length:0,Se=Ie.get(k),kt=m.state.lights;if(G===!0&&(ne===!0||E!==T)){const Tn=E===T&&k.id===b;ke.setState(k,E,Tn)}let Je=!1;k.version===Se.__version?(Se.needsLights&&Se.lightsStateVersion!==kt.state.version||Se.outputColorSpace!==ve||F.isBatchedMesh&&Se.batching===!1||!F.isBatchedMesh&&Se.batching===!0||F.isBatchedMesh&&Se.batchingColor===!0&&F.colorTexture===null||F.isBatchedMesh&&Se.batchingColor===!1&&F.colorTexture!==null||F.isInstancedMesh&&Se.instancing===!1||!F.isInstancedMesh&&Se.instancing===!0||F.isSkinnedMesh&&Se.skinning===!1||!F.isSkinnedMesh&&Se.skinning===!0||F.isInstancedMesh&&Se.instancingColor===!0&&F.instanceColor===null||F.isInstancedMesh&&Se.instancingColor===!1&&F.instanceColor!==null||F.isInstancedMesh&&Se.instancingMorph===!0&&F.morphTexture===null||F.isInstancedMesh&&Se.instancingMorph===!1&&F.morphTexture!==null||Se.envMap!==ye||k.fog===!0&&Se.fog!==se||Se.numClippingPlanes!==void 0&&(Se.numClippingPlanes!==ke.numPlanes||Se.numIntersection!==ke.numIntersection)||Se.vertexAlphas!==Re||Se.vertexTangents!==be||Se.morphTargets!==xe||Se.morphNormals!==Ze||Se.morphColors!==_t||Se.toneMapping!==vt||Se.morphTargetsCount!==Qe)&&(Je=!0):(Je=!0,Se.__version=k.version);let Un=Se.currentProgram;Je===!0&&(Un=ba(k,U,F));let Xr=!1,mn=!1,jc=!1;const Tt=Un.getUniforms(),Oi=Se.uniforms;if(Ee.useProgram(Un.program)&&(Xr=!0,mn=!0,jc=!0),k.id!==b&&(b=k.id,mn=!0),Xr||T!==E){Tt.setValue(P,"projectionMatrix",E.projectionMatrix),Tt.setValue(P,"viewMatrix",E.matrixWorldInverse);const Tn=Tt.map.cameraPosition;Tn!==void 0&&Tn.setValue(P,de.setFromMatrixPosition(E.matrixWorld)),rt.logarithmicDepthBuffer&&Tt.setValue(P,"logDepthBufFC",2/(Math.log(E.far+1)/Math.LN2)),(k.isMeshPhongMaterial||k.isMeshToonMaterial||k.isMeshLambertMaterial||k.isMeshBasicMaterial||k.isMeshStandardMaterial||k.isShaderMaterial)&&Tt.setValue(P,"isOrthographic",E.isOrthographicCamera===!0),T!==E&&(T=E,mn=!0,jc=!0)}if(F.isSkinnedMesh){Tt.setOptional(P,F,"bindMatrix"),Tt.setOptional(P,F,"bindMatrixInverse");const Tn=F.skeleton;Tn&&(Tn.boneTexture===null&&Tn.computeBoneTexture(),Tt.setValue(P,"boneTexture",Tn.boneTexture,Oe))}F.isBatchedMesh&&(Tt.setOptional(P,F,"batchingTexture"),Tt.setValue(P,"batchingTexture",F._matricesTexture,Oe),Tt.setOptional(P,F,"batchingIdTexture"),Tt.setValue(P,"batchingIdTexture",F._indirectTexture,Oe),Tt.setOptional(P,F,"batchingColorTexture"),F._colorsTexture!==null&&Tt.setValue(P,"batchingColorTexture",F._colorsTexture,Oe));const Yc=O.morphAttributes;if((Yc.position!==void 0||Yc.normal!==void 0||Yc.color!==void 0)&&Xe.update(F,O,Un),(mn||Se.receiveShadow!==F.receiveShadow)&&(Se.receiveShadow=F.receiveShadow,Tt.setValue(P,"receiveShadow",F.receiveShadow)),k.isMeshGouraudMaterial&&k.envMap!==null&&(Oi.envMap.value=ye,Oi.flipEnvMap.value=ye.isCubeTexture&&ye.isRenderTargetTexture===!1?-1:1),k.isMeshStandardMaterial&&k.envMap===null&&U.environment!==null&&(Oi.envMapIntensity.value=U.environmentIntensity),mn&&(Tt.setValue(P,"toneMappingExposure",v.toneMappingExposure),Se.needsLights&&ay(Oi,jc),se&&k.fog===!0&&Te.refreshFogUniforms(Oi,se),Te.refreshMaterialUniforms(Oi,k,$,W,m.state.transmissionRenderTarget[E.id]),Wl.upload(P,up(Se),Oi,Oe)),k.isShaderMaterial&&k.uniformsNeedUpdate===!0&&(Wl.upload(P,up(Se),Oi,Oe),k.uniformsNeedUpdate=!1),k.isSpriteMaterial&&Tt.setValue(P,"center",F.center),Tt.setValue(P,"modelViewMatrix",F.modelViewMatrix),Tt.setValue(P,"normalMatrix",F.normalMatrix),Tt.setValue(P,"modelMatrix",F.matrixWorld),k.isShaderMaterial||k.isRawShaderMaterial){const Tn=k.uniformsGroups;for(let Kc=0,cy=Tn.length;Kc<cy;Kc++){const fp=Tn[Kc];ft.update(fp,Un),ft.bind(fp,Un)}}return Un}function ay(E,U){E.ambientLightColor.needsUpdate=U,E.lightProbe.needsUpdate=U,E.directionalLights.needsUpdate=U,E.directionalLightShadows.needsUpdate=U,E.pointLights.needsUpdate=U,E.pointLightShadows.needsUpdate=U,E.spotLights.needsUpdate=U,E.spotLightShadows.needsUpdate=U,E.rectAreaLights.needsUpdate=U,E.hemisphereLights.needsUpdate=U}function ly(E){return E.isMeshLambertMaterial||E.isMeshToonMaterial||E.isMeshPhongMaterial||E.isMeshStandardMaterial||E.isShadowMaterial||E.isShaderMaterial&&E.lights===!0}this.getActiveCubeFace=function(){return R},this.getActiveMipmapLevel=function(){return A},this.getRenderTarget=function(){return w},this.setRenderTargetTextures=function(E,U,O){Ie.get(E.texture).__webglTexture=U,Ie.get(E.depthTexture).__webglTexture=O;const k=Ie.get(E);k.__hasExternalTextures=!0,k.__autoAllocateDepthBuffer=O===void 0,k.__autoAllocateDepthBuffer||et.has("WEBGL_multisampled_render_to_texture")===!0&&(console.warn("THREE.WebGLRenderer: Render-to-texture extension was disabled because an external texture was provided"),k.__useRenderToTexture=!1)},this.setRenderTargetFramebuffer=function(E,U){const O=Ie.get(E);O.__webglFramebuffer=U,O.__useDefaultFramebuffer=U===void 0},this.setRenderTarget=function(E,U=0,O=0){w=E,R=U,A=O;let k=!0,F=null,se=!1,fe=!1;if(E){const ye=Ie.get(E);if(ye.__useDefaultFramebuffer!==void 0)Ee.bindFramebuffer(P.FRAMEBUFFER,null),k=!1;else if(ye.__webglFramebuffer===void 0)Oe.setupRenderTarget(E);else if(ye.__hasExternalTextures)Oe.rebindTextures(E,Ie.get(E.texture).__webglTexture,Ie.get(E.depthTexture).__webglTexture);else if(E.depthBuffer){const xe=E.depthTexture;if(ye.__boundDepthTexture!==xe){if(xe!==null&&Ie.has(xe)&&(E.width!==xe.image.width||E.height!==xe.image.height))throw new Error("WebGLRenderTarget: Attached DepthTexture is initialized to the incorrect size.");Oe.setupDepthRenderbuffer(E)}}const Re=E.texture;(Re.isData3DTexture||Re.isDataArrayTexture||Re.isCompressedArrayTexture)&&(fe=!0);const be=Ie.get(E).__webglFramebuffer;E.isWebGLCubeRenderTarget?(Array.isArray(be[U])?F=be[U][O]:F=be[U],se=!0):E.samples>0&&Oe.useMultisampledRTT(E)===!1?F=Ie.get(E).__webglMultisampledFramebuffer:Array.isArray(be)?F=be[O]:F=be,M.copy(E.viewport),I.copy(E.scissor),V=E.scissorTest}else M.copy(Q).multiplyScalar($).floor(),I.copy(oe).multiplyScalar($).floor(),V=Me;if(Ee.bindFramebuffer(P.FRAMEBUFFER,F)&&k&&Ee.drawBuffers(E,F),Ee.viewport(M),Ee.scissor(I),Ee.setScissorTest(V),se){const ye=Ie.get(E.texture);P.framebufferTexture2D(P.FRAMEBUFFER,P.COLOR_ATTACHMENT0,P.TEXTURE_CUBE_MAP_POSITIVE_X+U,ye.__webglTexture,O)}else if(fe){const ye=Ie.get(E.texture),Re=U||0;P.framebufferTextureLayer(P.FRAMEBUFFER,P.COLOR_ATTACHMENT0,ye.__webglTexture,O||0,Re)}b=-1},this.readRenderTargetPixels=function(E,U,O,k,F,se,fe){if(!(E&&E.isWebGLRenderTarget)){console.error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not THREE.WebGLRenderTarget.");return}let ve=Ie.get(E).__webglFramebuffer;if(E.isWebGLCubeRenderTarget&&fe!==void 0&&(ve=ve[fe]),ve){Ee.bindFramebuffer(P.FRAMEBUFFER,ve);try{const ye=E.texture,Re=ye.format,be=ye.type;if(!rt.textureFormatReadable(Re)){console.error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not in RGBA or implementation defined format.");return}if(!rt.textureTypeReadable(be)){console.error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not in UnsignedByteType or implementation defined type.");return}U>=0&&U<=E.width-k&&O>=0&&O<=E.height-F&&P.readPixels(U,O,k,F,De.convert(Re),De.convert(be),se)}finally{const ye=w!==null?Ie.get(w).__webglFramebuffer:null;Ee.bindFramebuffer(P.FRAMEBUFFER,ye)}}},this.readRenderTargetPixelsAsync=async function(E,U,O,k,F,se,fe){if(!(E&&E.isWebGLRenderTarget))throw new Error("THREE.WebGLRenderer.readRenderTargetPixels: renderTarget is not THREE.WebGLRenderTarget.");let ve=Ie.get(E).__webglFramebuffer;if(E.isWebGLCubeRenderTarget&&fe!==void 0&&(ve=ve[fe]),ve){Ee.bindFramebuffer(P.FRAMEBUFFER,ve);try{const ye=E.texture,Re=ye.format,be=ye.type;if(!rt.textureFormatReadable(Re))throw new Error("THREE.WebGLRenderer.readRenderTargetPixelsAsync: renderTarget is not in RGBA or implementation defined format.");if(!rt.textureTypeReadable(be))throw new Error("THREE.WebGLRenderer.readRenderTargetPixelsAsync: renderTarget is not in UnsignedByteType or implementation defined type.");if(U>=0&&U<=E.width-k&&O>=0&&O<=E.height-F){const xe=P.createBuffer();P.bindBuffer(P.PIXEL_PACK_BUFFER,xe),P.bufferData(P.PIXEL_PACK_BUFFER,se.byteLength,P.STREAM_READ),P.readPixels(U,O,k,F,De.convert(Re),De.convert(be),0),P.flush();const Ze=P.fenceSync(P.SYNC_GPU_COMMANDS_COMPLETE,0);await fE(P,Ze,4);try{P.bindBuffer(P.PIXEL_PACK_BUFFER,xe),P.getBufferSubData(P.PIXEL_PACK_BUFFER,0,se)}finally{P.deleteBuffer(xe),P.deleteSync(Ze)}return se}}finally{const ye=w!==null?Ie.get(w).__webglFramebuffer:null;Ee.bindFramebuffer(P.FRAMEBUFFER,ye)}}},this.copyFramebufferToTexture=function(E,U=null,O=0){E.isTexture!==!0&&(Fs("WebGLRenderer: copyFramebufferToTexture function signature has changed."),U=arguments[0]||null,E=arguments[1]);const k=Math.pow(2,-O),F=Math.floor(E.image.width*k),se=Math.floor(E.image.height*k),fe=U!==null?U.x:0,ve=U!==null?U.y:0;Oe.setTexture2D(E,0),P.copyTexSubImage2D(P.TEXTURE_2D,O,0,0,fe,ve,F,se),Ee.unbindTexture()},this.copyTextureToTexture=function(E,U,O=null,k=null,F=0){E.isTexture!==!0&&(Fs("WebGLRenderer: copyTextureToTexture function signature has changed."),k=arguments[0]||null,E=arguments[1],U=arguments[2],F=arguments[3]||0,O=null);let se,fe,ve,ye,Re,be;O!==null?(se=O.max.x-O.min.x,fe=O.max.y-O.min.y,ve=O.min.x,ye=O.min.y):(se=E.image.width,fe=E.image.height,ve=0,ye=0),k!==null?(Re=k.x,be=k.y):(Re=0,be=0);const xe=De.convert(U.format),Ze=De.convert(U.type);Oe.setTexture2D(U,0),P.pixelStorei(P.UNPACK_FLIP_Y_WEBGL,U.flipY),P.pixelStorei(P.UNPACK_PREMULTIPLY_ALPHA_WEBGL,U.premultiplyAlpha),P.pixelStorei(P.UNPACK_ALIGNMENT,U.unpackAlignment);const _t=P.getParameter(P.UNPACK_ROW_LENGTH),vt=P.getParameter(P.UNPACK_IMAGE_HEIGHT),pn=P.getParameter(P.UNPACK_SKIP_PIXELS),Qe=P.getParameter(P.UNPACK_SKIP_ROWS),Se=P.getParameter(P.UNPACK_SKIP_IMAGES),kt=E.isCompressedTexture?E.mipmaps[F]:E.image;P.pixelStorei(P.UNPACK_ROW_LENGTH,kt.width),P.pixelStorei(P.UNPACK_IMAGE_HEIGHT,kt.height),P.pixelStorei(P.UNPACK_SKIP_PIXELS,ve),P.pixelStorei(P.UNPACK_SKIP_ROWS,ye),E.isDataTexture?P.texSubImage2D(P.TEXTURE_2D,F,Re,be,se,fe,xe,Ze,kt.data):E.isCompressedTexture?P.compressedTexSubImage2D(P.TEXTURE_2D,F,Re,be,kt.width,kt.height,xe,kt.data):P.texSubImage2D(P.TEXTURE_2D,F,Re,be,se,fe,xe,Ze,kt),P.pixelStorei(P.UNPACK_ROW_LENGTH,_t),P.pixelStorei(P.UNPACK_IMAGE_HEIGHT,vt),P.pixelStorei(P.UNPACK_SKIP_PIXELS,pn),P.pixelStorei(P.UNPACK_SKIP_ROWS,Qe),P.pixelStorei(P.UNPACK_SKIP_IMAGES,Se),F===0&&U.generateMipmaps&&P.generateMipmap(P.TEXTURE_2D),Ee.unbindTexture()},this.copyTextureToTexture3D=function(E,U,O=null,k=null,F=0){E.isTexture!==!0&&(Fs("WebGLRenderer: copyTextureToTexture3D function signature has changed."),O=arguments[0]||null,k=arguments[1]||null,E=arguments[2],U=arguments[3],F=arguments[4]||0);let se,fe,ve,ye,Re,be,xe,Ze,_t;const vt=E.isCompressedTexture?E.mipmaps[F]:E.image;O!==null?(se=O.max.x-O.min.x,fe=O.max.y-O.min.y,ve=O.max.z-O.min.z,ye=O.min.x,Re=O.min.y,be=O.min.z):(se=vt.width,fe=vt.height,ve=vt.depth,ye=0,Re=0,be=0),k!==null?(xe=k.x,Ze=k.y,_t=k.z):(xe=0,Ze=0,_t=0);const pn=De.convert(U.format),Qe=De.convert(U.type);let Se;if(U.isData3DTexture)Oe.setTexture3D(U,0),Se=P.TEXTURE_3D;else if(U.isDataArrayTexture||U.isCompressedArrayTexture)Oe.setTexture2DArray(U,0),Se=P.TEXTURE_2D_ARRAY;else{console.warn("THREE.WebGLRenderer.copyTextureToTexture3D: only supports THREE.DataTexture3D and THREE.DataTexture2DArray.");return}P.pixelStorei(P.UNPACK_FLIP_Y_WEBGL,U.flipY),P.pixelStorei(P.UNPACK_PREMULTIPLY_ALPHA_WEBGL,U.premultiplyAlpha),P.pixelStorei(P.UNPACK_ALIGNMENT,U.unpackAlignment);const kt=P.getParameter(P.UNPACK_ROW_LENGTH),Je=P.getParameter(P.UNPACK_IMAGE_HEIGHT),Un=P.getParameter(P.UNPACK_SKIP_PIXELS),Xr=P.getParameter(P.UNPACK_SKIP_ROWS),mn=P.getParameter(P.UNPACK_SKIP_IMAGES);P.pixelStorei(P.UNPACK_ROW_LENGTH,vt.width),P.pixelStorei(P.UNPACK_IMAGE_HEIGHT,vt.height),P.pixelStorei(P.UNPACK_SKIP_PIXELS,ye),P.pixelStorei(P.UNPACK_SKIP_ROWS,Re),P.pixelStorei(P.UNPACK_SKIP_IMAGES,be),E.isDataTexture||E.isData3DTexture?P.texSubImage3D(Se,F,xe,Ze,_t,se,fe,ve,pn,Qe,vt.data):U.isCompressedArrayTexture?P.compressedTexSubImage3D(Se,F,xe,Ze,_t,se,fe,ve,pn,vt.data):P.texSubImage3D(Se,F,xe,Ze,_t,se,fe,ve,pn,Qe,vt),P.pixelStorei(P.UNPACK_ROW_LENGTH,kt),P.pixelStorei(P.UNPACK_IMAGE_HEIGHT,Je),P.pixelStorei(P.UNPACK_SKIP_PIXELS,Un),P.pixelStorei(P.UNPACK_SKIP_ROWS,Xr),P.pixelStorei(P.UNPACK_SKIP_IMAGES,mn),F===0&&U.generateMipmaps&&P.generateMipmap(Se),Ee.unbindTexture()},this.initRenderTarget=function(E){Ie.get(E).__webglFramebuffer===void 0&&Oe.setupRenderTarget(E)},this.initTexture=function(E){E.isCubeTexture?Oe.setTextureCube(E,0):E.isData3DTexture?Oe.setTexture3D(E,0):E.isDataArrayTexture||E.isCompressedArrayTexture?Oe.setTexture2DArray(E,0):Oe.setTexture2D(E,0),Ee.unbindTexture()},this.resetState=function(){R=0,A=0,w=null,Ee.reset(),He.reset()},typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("observe",{detail:this}))}get coordinateSystem(){return wi}get outputColorSpace(){return this._outputColorSpace}set outputColorSpace(e){this._outputColorSpace=e;const t=this.getContext();t.drawingBufferColorSpace=e===Xh?"display-p3":"srgb",t.unpackColorSpace=qe.workingColorSpace===Hc?"display-p3":"srgb"}}class MR extends dt{constructor(){super(),this.isScene=!0,this.type="Scene",this.background=null,this.environment=null,this.fog=null,this.backgroundBlurriness=0,this.backgroundIntensity=1,this.backgroundRotation=new ai,this.environmentIntensity=1,this.environmentRotation=new ai,this.overrideMaterial=null,typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("observe",{detail:this}))}copy(e,t){return super.copy(e,t),e.background!==null&&(this.background=e.background.clone()),e.environment!==null&&(this.environment=e.environment.clone()),e.fog!==null&&(this.fog=e.fog.clone()),this.backgroundBlurriness=e.backgroundBlurriness,this.backgroundIntensity=e.backgroundIntensity,this.backgroundRotation.copy(e.backgroundRotation),this.environmentIntensity=e.environmentIntensity,this.environmentRotation.copy(e.environmentRotation),e.overrideMaterial!==null&&(this.overrideMaterial=e.overrideMaterial.clone()),this.matrixAutoUpdate=e.matrixAutoUpdate,this}toJSON(e){const t=super.toJSON(e);return this.fog!==null&&(t.object.fog=this.fog.toJSON()),this.backgroundBlurriness>0&&(t.object.backgroundBlurriness=this.backgroundBlurriness),this.backgroundIntensity!==1&&(t.object.backgroundIntensity=this.backgroundIntensity),t.object.backgroundRotation=this.backgroundRotation.toArray(),this.environmentIntensity!==1&&(t.object.environmentIntensity=this.environmentIntensity),t.object.environmentRotation=this.environmentRotation.toArray(),t}}class Kv{constructor(e,t){this.isInterleavedBuffer=!0,this.array=e,this.stride=t,this.count=e!==void 0?e.length/t:0,this.usage=Df,this._updateRange={offset:0,count:-1},this.updateRanges=[],this.version=0,this.uuid=Kn()}onUploadCallback(){}set needsUpdate(e){e===!0&&this.version++}get updateRange(){return Fs("THREE.InterleavedBuffer: updateRange() is deprecated and will be removed in r169. Use addUpdateRange() instead."),this._updateRange}setUsage(e){return this.usage=e,this}addUpdateRange(e,t){this.updateRanges.push({start:e,count:t})}clearUpdateRanges(){this.updateRanges.length=0}copy(e){return this.array=new e.array.constructor(e.array),this.count=e.count,this.stride=e.stride,this.usage=e.usage,this}copyAt(e,t,i){e*=this.stride,i*=t.stride;for(let r=0,s=this.stride;r<s;r++)this.array[e+r]=t.array[i+r];return this}set(e,t=0){return this.array.set(e,t),this}clone(e){e.arrayBuffers===void 0&&(e.arrayBuffers={}),this.array.buffer._uuid===void 0&&(this.array.buffer._uuid=Kn()),e.arrayBuffers[this.array.buffer._uuid]===void 0&&(e.arrayBuffers[this.array.buffer._uuid]=this.array.slice(0).buffer);const t=new this.array.constructor(e.arrayBuffers[this.array.buffer._uuid]),i=new this.constructor(t,this.stride);return i.setUsage(this.usage),i}onUpload(e){return this.onUploadCallback=e,this}toJSON(e){return e.arrayBuffers===void 0&&(e.arrayBuffers={}),this.array.buffer._uuid===void 0&&(this.array.buffer._uuid=Kn()),e.arrayBuffers[this.array.buffer._uuid]===void 0&&(e.arrayBuffers[this.array.buffer._uuid]=Array.from(new Uint32Array(this.array.buffer))),{uuid:this.uuid,buffer:this.array.buffer._uuid,type:this.array.constructor.name,stride:this.stride}}}const qt=new L;class _a{constructor(e,t,i,r=!1){this.isInterleavedBufferAttribute=!0,this.name="",this.data=e,this.itemSize=t,this.offset=i,this.normalized=r}get count(){return this.data.count}get array(){return this.data.array}set needsUpdate(e){this.data.needsUpdate=e}applyMatrix4(e){for(let t=0,i=this.data.count;t<i;t++)qt.fromBufferAttribute(this,t),qt.applyMatrix4(e),this.setXYZ(t,qt.x,qt.y,qt.z);return this}applyNormalMatrix(e){for(let t=0,i=this.count;t<i;t++)qt.fromBufferAttribute(this,t),qt.applyNormalMatrix(e),this.setXYZ(t,qt.x,qt.y,qt.z);return this}transformDirection(e){for(let t=0,i=this.count;t<i;t++)qt.fromBufferAttribute(this,t),qt.transformDirection(e),this.setXYZ(t,qt.x,qt.y,qt.z);return this}getComponent(e,t){let i=this.array[e*this.data.stride+this.offset+t];return this.normalized&&(i=Gn(i,this.array)),i}setComponent(e,t,i){return this.normalized&&(i=tt(i,this.array)),this.data.array[e*this.data.stride+this.offset+t]=i,this}setX(e,t){return this.normalized&&(t=tt(t,this.array)),this.data.array[e*this.data.stride+this.offset]=t,this}setY(e,t){return this.normalized&&(t=tt(t,this.array)),this.data.array[e*this.data.stride+this.offset+1]=t,this}setZ(e,t){return this.normalized&&(t=tt(t,this.array)),this.data.array[e*this.data.stride+this.offset+2]=t,this}setW(e,t){return this.normalized&&(t=tt(t,this.array)),this.data.array[e*this.data.stride+this.offset+3]=t,this}getX(e){let t=this.data.array[e*this.data.stride+this.offset];return this.normalized&&(t=Gn(t,this.array)),t}getY(e){let t=this.data.array[e*this.data.stride+this.offset+1];return this.normalized&&(t=Gn(t,this.array)),t}getZ(e){let t=this.data.array[e*this.data.stride+this.offset+2];return this.normalized&&(t=Gn(t,this.array)),t}getW(e){let t=this.data.array[e*this.data.stride+this.offset+3];return this.normalized&&(t=Gn(t,this.array)),t}setXY(e,t,i){return e=e*this.data.stride+this.offset,this.normalized&&(t=tt(t,this.array),i=tt(i,this.array)),this.data.array[e+0]=t,this.data.array[e+1]=i,this}setXYZ(e,t,i,r){return e=e*this.data.stride+this.offset,this.normalized&&(t=tt(t,this.array),i=tt(i,this.array),r=tt(r,this.array)),this.data.array[e+0]=t,this.data.array[e+1]=i,this.data.array[e+2]=r,this}setXYZW(e,t,i,r,s){return e=e*this.data.stride+this.offset,this.normalized&&(t=tt(t,this.array),i=tt(i,this.array),r=tt(r,this.array),s=tt(s,this.array)),this.data.array[e+0]=t,this.data.array[e+1]=i,this.data.array[e+2]=r,this.data.array[e+3]=s,this}clone(e){if(e===void 0){console.log("THREE.InterleavedBufferAttribute.clone(): Cloning an interleaved buffer attribute will de-interleave buffer data.");const t=[];for(let i=0;i<this.count;i++){const r=i*this.data.stride+this.offset;for(let s=0;s<this.itemSize;s++)t.push(this.data.array[r+s])}return new Rt(new this.array.constructor(t),this.itemSize,this.normalized)}else return e.interleavedBuffers===void 0&&(e.interleavedBuffers={}),e.interleavedBuffers[this.data.uuid]===void 0&&(e.interleavedBuffers[this.data.uuid]=this.data.clone(e)),new _a(e.interleavedBuffers[this.data.uuid],this.itemSize,this.offset,this.normalized)}toJSON(e){if(e===void 0){console.log("THREE.InterleavedBufferAttribute.toJSON(): Serializing an interleaved buffer attribute will de-interleave buffer data.");const t=[];for(let i=0;i<this.count;i++){const r=i*this.data.stride+this.offset;for(let s=0;s<this.itemSize;s++)t.push(this.data.array[r+s])}return{itemSize:this.itemSize,type:this.array.constructor.name,array:t,normalized:this.normalized}}else return e.interleavedBuffers===void 0&&(e.interleavedBuffers={}),e.interleavedBuffers[this.data.uuid]===void 0&&(e.interleavedBuffers[this.data.uuid]=this.data.toJSON(e)),{isInterleavedBufferAttribute:!0,itemSize:this.itemSize,data:this.data.uuid,offset:this.offset,normalized:this.normalized}}}class Ff extends qn{constructor(e){super(),this.isSpriteMaterial=!0,this.type="SpriteMaterial",this.color=new Pe(16777215),this.map=null,this.alphaMap=null,this.rotation=0,this.sizeAttenuation=!0,this.transparent=!0,this.fog=!0,this.setValues(e)}copy(e){return super.copy(e),this.color.copy(e.color),this.map=e.map,this.alphaMap=e.alphaMap,this.rotation=e.rotation,this.sizeAttenuation=e.sizeAttenuation,this.fog=e.fog,this}}let us;const To=new L,ds=new L,fs=new L,hs=new we,wo=new we,qv=new Ue,pl=new L,Ao=new L,ml=new L,Mg=new we,Yu=new we,Eg=new we;class Tg extends dt{constructor(e=new Ff){if(super(),this.isSprite=!0,this.type="Sprite",us===void 0){us=new hn;const t=new Float32Array([-.5,-.5,0,0,0,.5,-.5,0,1,0,.5,.5,0,1,1,-.5,.5,0,0,1]),i=new Kv(t,5);us.setIndex([0,1,2,0,2,3]),us.setAttribute("position",new _a(i,3,0,!1)),us.setAttribute("uv",new _a(i,2,3,!1))}this.geometry=us,this.material=e,this.center=new we(.5,.5)}raycast(e,t){e.camera===null&&console.error('THREE.Sprite: "Raycaster.camera" needs to be set in order to raycast against sprites.'),ds.setFromMatrixScale(this.matrixWorld),qv.copy(e.camera.matrixWorld),this.modelViewMatrix.multiplyMatrices(e.camera.matrixWorldInverse,this.matrixWorld),fs.setFromMatrixPosition(this.modelViewMatrix),e.camera.isPerspectiveCamera&&this.material.sizeAttenuation===!1&&ds.multiplyScalar(-fs.z);const i=this.material.rotation;let r,s;i!==0&&(s=Math.cos(i),r=Math.sin(i));const o=this.center;gl(pl.set(-.5,-.5,0),fs,o,ds,r,s),gl(Ao.set(.5,-.5,0),fs,o,ds,r,s),gl(ml.set(.5,.5,0),fs,o,ds,r,s),Mg.set(0,0),Yu.set(1,0),Eg.set(1,1);let a=e.ray.intersectTriangle(pl,Ao,ml,!1,To);if(a===null&&(gl(Ao.set(-.5,.5,0),fs,o,ds,r,s),Yu.set(0,1),a=e.ray.intersectTriangle(pl,ml,Ao,!1,To),a===null))return;const l=e.ray.origin.distanceTo(To);l<e.near||l>e.far||t.push({distance:l,point:To.clone(),uv:Wn.getInterpolation(To,pl,Ao,ml,Mg,Yu,Eg,new we),face:null,object:this})}copy(e,t){return super.copy(e,t),e.center!==void 0&&this.center.copy(e.center),this.material=e.material,this}}function gl(n,e,t,i,r,s){hs.subVectors(n,t).addScalar(.5).multiply(i),r!==void 0?(wo.x=s*hs.x-r*hs.y,wo.y=r*hs.x+s*hs.y):wo.copy(hs),n.copy(e),n.x+=wo.x,n.y+=wo.y,n.applyMatrix4(qv)}const wg=new L,Ag=new st,Rg=new st,ER=new L,Cg=new Ue,_l=new L,Ku=new li,bg=new Ue,qu=new wa;class TR extends tn{constructor(e,t){super(e,t),this.isSkinnedMesh=!0,this.type="SkinnedMesh",this.bindMode=Am,this.bindMatrix=new Ue,this.bindMatrixInverse=new Ue,this.boundingBox=null,this.boundingSphere=null}computeBoundingBox(){const e=this.geometry;this.boundingBox===null&&(this.boundingBox=new Fi),this.boundingBox.makeEmpty();const t=e.getAttribute("position");for(let i=0;i<t.count;i++)this.getVertexPosition(i,_l),this.boundingBox.expandByPoint(_l)}computeBoundingSphere(){const e=this.geometry;this.boundingSphere===null&&(this.boundingSphere=new li),this.boundingSphere.makeEmpty();const t=e.getAttribute("position");for(let i=0;i<t.count;i++)this.getVertexPosition(i,_l),this.boundingSphere.expandByPoint(_l)}copy(e,t){return super.copy(e,t),this.bindMode=e.bindMode,this.bindMatrix.copy(e.bindMatrix),this.bindMatrixInverse.copy(e.bindMatrixInverse),this.skeleton=e.skeleton,e.boundingBox!==null&&(this.boundingBox=e.boundingBox.clone()),e.boundingSphere!==null&&(this.boundingSphere=e.boundingSphere.clone()),this}raycast(e,t){const i=this.material,r=this.matrixWorld;i!==void 0&&(this.boundingSphere===null&&this.computeBoundingSphere(),Ku.copy(this.boundingSphere),Ku.applyMatrix4(r),e.ray.intersectsSphere(Ku)!==!1&&(bg.copy(r).invert(),qu.copy(e.ray).applyMatrix4(bg),!(this.boundingBox!==null&&qu.intersectsBox(this.boundingBox)===!1)&&this._computeIntersections(e,t,qu)))}getVertexPosition(e,t){return super.getVertexPosition(e,t),this.applyBoneTransform(e,t),t}bind(e,t){this.skeleton=e,t===void 0&&(this.updateMatrixWorld(!0),this.skeleton.calculateInverses(),t=this.matrixWorld),this.bindMatrix.copy(t),this.bindMatrixInverse.copy(t).invert()}pose(){this.skeleton.pose()}normalizeSkinWeights(){const e=new st,t=this.geometry.attributes.skinWeight;for(let i=0,r=t.count;i<r;i++){e.fromBufferAttribute(t,i);const s=1/e.manhattanLength();s!==1/0?e.multiplyScalar(s):e.set(1,0,0,0),t.setXYZW(i,e.x,e.y,e.z,e.w)}}updateMatrixWorld(e){super.updateMatrixWorld(e),this.bindMode===Am?this.bindMatrixInverse.copy(this.matrixWorld).invert():this.bindMode===UM?this.bindMatrixInverse.copy(this.bindMatrix).invert():console.warn("THREE.SkinnedMesh: Unrecognized bindMode: "+this.bindMode)}applyBoneTransform(e,t){const i=this.skeleton,r=this.geometry;Ag.fromBufferAttribute(r.attributes.skinIndex,e),Rg.fromBufferAttribute(r.attributes.skinWeight,e),wg.copy(t).applyMatrix4(this.bindMatrix),t.set(0,0,0);for(let s=0;s<4;s++){const o=Rg.getComponent(s);if(o!==0){const a=Ag.getComponent(s);Cg.multiplyMatrices(i.bones[a].matrixWorld,i.boneInverses[a]),t.addScaledVector(ER.copy(wg).applyMatrix4(Cg),o)}}return t.applyMatrix4(this.bindMatrixInverse)}}class $v extends dt{constructor(){super(),this.isBone=!0,this.type="Bone"}}class Zv extends Pt{constructor(e=null,t=1,i=1,r,s,o,a,l,c=en,u=en,d,f){super(null,o,a,l,c,u,r,s,d,f),this.isDataTexture=!0,this.image={data:e,width:t,height:i},this.generateMipmaps=!1,this.flipY=!1,this.unpackAlignment=1}}const Pg=new Ue,wR=new Ue;class Zh{constructor(e=[],t=[]){this.uuid=Kn(),this.bones=e.slice(0),this.boneInverses=t,this.boneMatrices=null,this.boneTexture=null,this.init()}init(){const e=this.bones,t=this.boneInverses;if(this.boneMatrices=new Float32Array(e.length*16),t.length===0)this.calculateInverses();else if(e.length!==t.length){console.warn("THREE.Skeleton: Number of inverse bone matrices does not match amount of bones."),this.boneInverses=[];for(let i=0,r=this.bones.length;i<r;i++)this.boneInverses.push(new Ue)}}calculateInverses(){this.boneInverses.length=0;for(let e=0,t=this.bones.length;e<t;e++){const i=new Ue;this.bones[e]&&i.copy(this.bones[e].matrixWorld).invert(),this.boneInverses.push(i)}}pose(){for(let e=0,t=this.bones.length;e<t;e++){const i=this.bones[e];i&&i.matrixWorld.copy(this.boneInverses[e]).invert()}for(let e=0,t=this.bones.length;e<t;e++){const i=this.bones[e];i&&(i.parent&&i.parent.isBone?(i.matrix.copy(i.parent.matrixWorld).invert(),i.matrix.multiply(i.matrixWorld)):i.matrix.copy(i.matrixWorld),i.matrix.decompose(i.position,i.quaternion,i.scale))}}update(){const e=this.bones,t=this.boneInverses,i=this.boneMatrices,r=this.boneTexture;for(let s=0,o=e.length;s<o;s++){const a=e[s]?e[s].matrixWorld:wR;Pg.multiplyMatrices(a,t[s]),Pg.toArray(i,s*16)}r!==null&&(r.needsUpdate=!0)}clone(){return new Zh(this.bones,this.boneInverses)}computeBoneTexture(){let e=Math.sqrt(this.bones.length*4);e=Math.ceil(e/4)*4,e=Math.max(e,4);const t=new Float32Array(e*e*4);t.set(this.boneMatrices);const i=new Zv(t,e,e,Pn,Xn);return i.needsUpdate=!0,this.boneMatrices=t,this.boneTexture=i,this}getBoneByName(e){for(let t=0,i=this.bones.length;t<i;t++){const r=this.bones[t];if(r.name===e)return r}}dispose(){this.boneTexture!==null&&(this.boneTexture.dispose(),this.boneTexture=null)}fromJSON(e,t){this.uuid=e.uuid;for(let i=0,r=e.bones.length;i<r;i++){const s=e.bones[i];let o=t[s];o===void 0&&(console.warn("THREE.Skeleton: No bone found with UUID:",s),o=new $v),this.bones.push(o),this.boneInverses.push(new Ue().fromArray(e.boneInverses[i]))}return this.init(),this}toJSON(){const e={metadata:{version:4.6,type:"Skeleton",generator:"Skeleton.toJSON"},bones:[],boneInverses:[]};e.uuid=this.uuid;const t=this.bones,i=this.boneInverses;for(let r=0,s=t.length;r<s;r++){const o=t[r];e.bones.push(o.uuid);const a=i[r];e.boneInverses.push(a.toArray())}return e}}class Of extends Rt{constructor(e,t,i,r=1){super(e,t,i),this.isInstancedBufferAttribute=!0,this.meshPerAttribute=r}copy(e){return super.copy(e),this.meshPerAttribute=e.meshPerAttribute,this}toJSON(){const e=super.toJSON();return e.meshPerAttribute=this.meshPerAttribute,e.isInstancedBufferAttribute=!0,e}}const ps=new Ue,Lg=new Ue,vl=[],Ig=new Fi,AR=new Ue,Ro=new tn,Co=new li;class RR extends tn{constructor(e,t,i){super(e,t),this.isInstancedMesh=!0,this.instanceMatrix=new Of(new Float32Array(i*16),16),this.instanceColor=null,this.morphTexture=null,this.count=i,this.boundingBox=null,this.boundingSphere=null;for(let r=0;r<i;r++)this.setMatrixAt(r,AR)}computeBoundingBox(){const e=this.geometry,t=this.count;this.boundingBox===null&&(this.boundingBox=new Fi),e.boundingBox===null&&e.computeBoundingBox(),this.boundingBox.makeEmpty();for(let i=0;i<t;i++)this.getMatrixAt(i,ps),Ig.copy(e.boundingBox).applyMatrix4(ps),this.boundingBox.union(Ig)}computeBoundingSphere(){const e=this.geometry,t=this.count;this.boundingSphere===null&&(this.boundingSphere=new li),e.boundingSphere===null&&e.computeBoundingSphere(),this.boundingSphere.makeEmpty();for(let i=0;i<t;i++)this.getMatrixAt(i,ps),Co.copy(e.boundingSphere).applyMatrix4(ps),this.boundingSphere.union(Co)}copy(e,t){return super.copy(e,t),this.instanceMatrix.copy(e.instanceMatrix),e.morphTexture!==null&&(this.morphTexture=e.morphTexture.clone()),e.instanceColor!==null&&(this.instanceColor=e.instanceColor.clone()),this.count=e.count,e.boundingBox!==null&&(this.boundingBox=e.boundingBox.clone()),e.boundingSphere!==null&&(this.boundingSphere=e.boundingSphere.clone()),this}getColorAt(e,t){t.fromArray(this.instanceColor.array,e*3)}getMatrixAt(e,t){t.fromArray(this.instanceMatrix.array,e*16)}getMorphAt(e,t){const i=t.morphTargetInfluences,r=this.morphTexture.source.data.data,s=i.length+1,o=e*s+1;for(let a=0;a<i.length;a++)i[a]=r[o+a]}raycast(e,t){const i=this.matrixWorld,r=this.count;if(Ro.geometry=this.geometry,Ro.material=this.material,Ro.material!==void 0&&(this.boundingSphere===null&&this.computeBoundingSphere(),Co.copy(this.boundingSphere),Co.applyMatrix4(i),e.ray.intersectsSphere(Co)!==!1))for(let s=0;s<r;s++){this.getMatrixAt(s,ps),Lg.multiplyMatrices(i,ps),Ro.matrixWorld=Lg,Ro.raycast(e,vl);for(let o=0,a=vl.length;o<a;o++){const l=vl[o];l.instanceId=s,l.object=this,t.push(l)}vl.length=0}}setColorAt(e,t){this.instanceColor===null&&(this.instanceColor=new Of(new Float32Array(this.instanceMatrix.count*3).fill(1),3)),t.toArray(this.instanceColor.array,e*3)}setMatrixAt(e,t){t.toArray(this.instanceMatrix.array,e*16)}setMorphAt(e,t){const i=t.morphTargetInfluences,r=i.length+1;this.morphTexture===null&&(this.morphTexture=new Zv(new Float32Array(r*this.count),r,this.count,Hh,Xn));const s=this.morphTexture.source.data.data;let o=0;for(let c=0;c<i.length;c++)o+=i[c];const a=this.geometry.morphTargetsRelative?1:1-o,l=r*e;s[l]=a,s.set(i,l+1)}updateMorphTargets(){}dispose(){return this.dispatchEvent({type:"dispose"}),this.morphTexture!==null&&(this.morphTexture.dispose(),this.morphTexture=null),this}}class Qh extends qn{constructor(e){super(),this.isLineBasicMaterial=!0,this.type="LineBasicMaterial",this.color=new Pe(16777215),this.map=null,this.linewidth=1,this.linecap="round",this.linejoin="round",this.fog=!0,this.setValues(e)}copy(e){return super.copy(e),this.color.copy(e.color),this.map=e.map,this.linewidth=e.linewidth,this.linecap=e.linecap,this.linejoin=e.linejoin,this.fog=e.fog,this}}const Mc=new L,Ec=new L,Ng=new Ue,bo=new wa,yl=new li,$u=new L,Dg=new L;class Wc extends dt{constructor(e=new hn,t=new Qh){super(),this.isLine=!0,this.type="Line",this.geometry=e,this.material=t,this.updateMorphTargets()}copy(e,t){return super.copy(e,t),this.material=Array.isArray(e.material)?e.material.slice():e.material,this.geometry=e.geometry,this}computeLineDistances(){const e=this.geometry;if(e.index===null){const t=e.attributes.position,i=[0];for(let r=1,s=t.count;r<s;r++)Mc.fromBufferAttribute(t,r-1),Ec.fromBufferAttribute(t,r),i[r]=i[r-1],i[r]+=Mc.distanceTo(Ec);e.setAttribute("lineDistance",new In(i,1))}else console.warn("THREE.Line.computeLineDistances(): Computation only possible with non-indexed BufferGeometry.");return this}raycast(e,t){const i=this.geometry,r=this.matrixWorld,s=e.params.Line.threshold,o=i.drawRange;if(i.boundingSphere===null&&i.computeBoundingSphere(),yl.copy(i.boundingSphere),yl.applyMatrix4(r),yl.radius+=s,e.ray.intersectsSphere(yl)===!1)return;Ng.copy(r).invert(),bo.copy(e.ray).applyMatrix4(Ng);const a=s/((this.scale.x+this.scale.y+this.scale.z)/3),l=a*a,c=this.isLineSegments?2:1,u=i.index,f=i.attributes.position;if(u!==null){const p=Math.max(0,o.start),g=Math.min(u.count,o.start+o.count);for(let y=p,m=g-1;y<m;y+=c){const h=u.getX(y),_=u.getX(y+1),v=xl(this,e,bo,l,h,_);v&&t.push(v)}if(this.isLineLoop){const y=u.getX(g-1),m=u.getX(p),h=xl(this,e,bo,l,y,m);h&&t.push(h)}}else{const p=Math.max(0,o.start),g=Math.min(f.count,o.start+o.count);for(let y=p,m=g-1;y<m;y+=c){const h=xl(this,e,bo,l,y,y+1);h&&t.push(h)}if(this.isLineLoop){const y=xl(this,e,bo,l,g-1,p);y&&t.push(y)}}}updateMorphTargets(){const t=this.geometry.morphAttributes,i=Object.keys(t);if(i.length>0){const r=t[i[0]];if(r!==void 0){this.morphTargetInfluences=[],this.morphTargetDictionary={};for(let s=0,o=r.length;s<o;s++){const a=r[s].name||String(s);this.morphTargetInfluences.push(0),this.morphTargetDictionary[a]=s}}}}}function xl(n,e,t,i,r,s){const o=n.geometry.attributes.position;if(Mc.fromBufferAttribute(o,r),Ec.fromBufferAttribute(o,s),t.distanceSqToSegment(Mc,Ec,$u,Dg)>i)return;$u.applyMatrix4(n.matrixWorld);const l=e.ray.origin.distanceTo($u);if(!(l<e.near||l>e.far))return{distance:l,point:Dg.clone().applyMatrix4(n.matrixWorld),index:r,face:null,faceIndex:null,object:n}}const Ug=new L,Fg=new L;class CR extends Wc{constructor(e,t){super(e,t),this.isLineSegments=!0,this.type="LineSegments"}computeLineDistances(){const e=this.geometry;if(e.index===null){const t=e.attributes.position,i=[];for(let r=0,s=t.count;r<s;r+=2)Ug.fromBufferAttribute(t,r),Fg.fromBufferAttribute(t,r+1),i[r]=r===0?0:i[r-1],i[r+1]=i[r]+Ug.distanceTo(Fg);e.setAttribute("lineDistance",new In(i,1))}else console.warn("THREE.LineSegments.computeLineDistances(): Computation only possible with non-indexed BufferGeometry.");return this}}class bR extends Wc{constructor(e,t){super(e,t),this.isLineLoop=!0,this.type="LineLoop"}}class Qv extends qn{constructor(e){super(),this.isPointsMaterial=!0,this.type="PointsMaterial",this.color=new Pe(16777215),this.map=null,this.alphaMap=null,this.size=1,this.sizeAttenuation=!0,this.fog=!0,this.setValues(e)}copy(e){return super.copy(e),this.color.copy(e.color),this.map=e.map,this.alphaMap=e.alphaMap,this.size=e.size,this.sizeAttenuation=e.sizeAttenuation,this.fog=e.fog,this}}const Og=new Ue,kf=new wa,Sl=new li,Ml=new L;class Jv extends dt{constructor(e=new hn,t=new Qv){super(),this.isPoints=!0,this.type="Points",this.geometry=e,this.material=t,this.updateMorphTargets()}copy(e,t){return super.copy(e,t),this.material=Array.isArray(e.material)?e.material.slice():e.material,this.geometry=e.geometry,this}raycast(e,t){const i=this.geometry,r=this.matrixWorld,s=e.params.Points.threshold,o=i.drawRange;if(i.boundingSphere===null&&i.computeBoundingSphere(),Sl.copy(i.boundingSphere),Sl.applyMatrix4(r),Sl.radius+=s,e.ray.intersectsSphere(Sl)===!1)return;Og.copy(r).invert(),kf.copy(e.ray).applyMatrix4(Og);const a=s/((this.scale.x+this.scale.y+this.scale.z)/3),l=a*a,c=i.index,d=i.attributes.position;if(c!==null){const f=Math.max(0,o.start),p=Math.min(c.count,o.start+o.count);for(let g=f,y=p;g<y;g++){const m=c.getX(g);Ml.fromBufferAttribute(d,m),kg(Ml,m,l,r,e,t,this)}}else{const f=Math.max(0,o.start),p=Math.min(d.count,o.start+o.count);for(let g=f,y=p;g<y;g++)Ml.fromBufferAttribute(d,g),kg(Ml,g,l,r,e,t,this)}}updateMorphTargets(){const t=this.geometry.morphAttributes,i=Object.keys(t);if(i.length>0){const r=t[i[0]];if(r!==void 0){this.morphTargetInfluences=[],this.morphTargetDictionary={};for(let s=0,o=r.length;s<o;s++){const a=r[s].name||String(s);this.morphTargetInfluences.push(0),this.morphTargetDictionary[a]=s}}}}}function kg(n,e,t,i,r,s,o){const a=kf.distanceSqToPoint(n);if(a<t){const l=new L;kf.closestPointToPoint(n,l),l.applyMatrix4(i);const c=r.ray.origin.distanceTo(l);if(c<r.near||c>r.far)return;s.push({distance:c,distanceToRay:Math.sqrt(a),point:l,index:e,face:null,object:o})}}class Bg extends Pt{constructor(e,t,i,r,s,o,a,l,c){super(e,t,i,r,s,o,a,l,c),this.isCanvasTexture=!0,this.needsUpdate=!0}}class PR{constructor(){this.type="Curve",this.arcLengthDivisions=200}getPoint(){return console.warn("THREE.Curve: .getPoint() not implemented."),null}getPointAt(e,t){const i=this.getUtoTmapping(e);return this.getPoint(i,t)}getPoints(e=5){const t=[];for(let i=0;i<=e;i++)t.push(this.getPoint(i/e));return t}getSpacedPoints(e=5){const t=[];for(let i=0;i<=e;i++)t.push(this.getPointAt(i/e));return t}getLength(){const e=this.getLengths();return e[e.length-1]}getLengths(e=this.arcLengthDivisions){if(this.cacheArcLengths&&this.cacheArcLengths.length===e+1&&!this.needsUpdate)return this.cacheArcLengths;this.needsUpdate=!1;const t=[];let i,r=this.getPoint(0),s=0;t.push(0);for(let o=1;o<=e;o++)i=this.getPoint(o/e),s+=i.distanceTo(r),t.push(s),r=i;return this.cacheArcLengths=t,t}updateArcLengths(){this.needsUpdate=!0,this.getLengths()}getUtoTmapping(e,t){const i=this.getLengths();let r=0;const s=i.length;let o;t?o=t:o=e*i[s-1];let a=0,l=s-1,c;for(;a<=l;)if(r=Math.floor(a+(l-a)/2),c=i[r]-o,c<0)a=r+1;else if(c>0)l=r-1;else{l=r;break}if(r=l,i[r]===o)return r/(s-1);const u=i[r],f=i[r+1]-u,p=(o-u)/f;return(r+p)/(s-1)}getTangent(e,t){let r=e-1e-4,s=e+1e-4;r<0&&(r=0),s>1&&(s=1);const o=this.getPoint(r),a=this.getPoint(s),l=t||(o.isVector2?new we:new L);return l.copy(a).sub(o).normalize(),l}getTangentAt(e,t){const i=this.getUtoTmapping(e);return this.getTangent(i,t)}computeFrenetFrames(e,t){const i=new L,r=[],s=[],o=[],a=new L,l=new Ue;for(let p=0;p<=e;p++){const g=p/e;r[p]=this.getTangentAt(g,new L)}s[0]=new L,o[0]=new L;let c=Number.MAX_VALUE;const u=Math.abs(r[0].x),d=Math.abs(r[0].y),f=Math.abs(r[0].z);u<=c&&(c=u,i.set(1,0,0)),d<=c&&(c=d,i.set(0,1,0)),f<=c&&i.set(0,0,1),a.crossVectors(r[0],i).normalize(),s[0].crossVectors(r[0],a),o[0].crossVectors(r[0],s[0]);for(let p=1;p<=e;p++){if(s[p]=s[p-1].clone(),o[p]=o[p-1].clone(),a.crossVectors(r[p-1],r[p]),a.length()>Number.EPSILON){a.normalize();const g=Math.acos(Ut(r[p-1].dot(r[p]),-1,1));s[p].applyMatrix4(l.makeRotationAxis(a,g))}o[p].crossVectors(r[p],s[p])}if(t===!0){let p=Math.acos(Ut(s[0].dot(s[e]),-1,1));p/=e,r[0].dot(a.crossVectors(s[0],s[e]))>0&&(p=-p);for(let g=1;g<=e;g++)s[g].applyMatrix4(l.makeRotationAxis(r[g],p*g)),o[g].crossVectors(r[g],s[g])}return{tangents:r,normals:s,binormals:o}}clone(){return new this.constructor().copy(this)}copy(e){return this.arcLengthDivisions=e.arcLengthDivisions,this}toJSON(){const e={metadata:{version:4.6,type:"Curve",generator:"Curve.toJSON"}};return e.arcLengthDivisions=this.arcLengthDivisions,e.type=this.type,e}fromJSON(e){return this.arcLengthDivisions=e.arcLengthDivisions,this}}function Jh(){let n=0,e=0,t=0,i=0;function r(s,o,a,l){n=s,e=a,t=-3*s+3*o-2*a-l,i=2*s-2*o+a+l}return{initCatmullRom:function(s,o,a,l,c){r(o,a,c*(a-s),c*(l-o))},initNonuniformCatmullRom:function(s,o,a,l,c,u,d){let f=(o-s)/c-(a-s)/(c+u)+(a-o)/u,p=(a-o)/u-(l-o)/(u+d)+(l-a)/d;f*=u,p*=u,r(o,a,f,p)},calc:function(s){const o=s*s,a=o*s;return n+e*s+t*o+i*a}}}const El=new L,Zu=new Jh,Qu=new Jh,Ju=new Jh;class LR extends PR{constructor(e=[],t=!1,i="centripetal",r=.5){super(),this.isCatmullRomCurve3=!0,this.type="CatmullRomCurve3",this.points=e,this.closed=t,this.curveType=i,this.tension=r}getPoint(e,t=new L){const i=t,r=this.points,s=r.length,o=(s-(this.closed?0:1))*e;let a=Math.floor(o),l=o-a;this.closed?a+=a>0?0:(Math.floor(Math.abs(a)/s)+1)*s:l===0&&a===s-1&&(a=s-2,l=1);let c,u;this.closed||a>0?c=r[(a-1)%s]:(El.subVectors(r[0],r[1]).add(r[0]),c=El);const d=r[a%s],f=r[(a+1)%s];if(this.closed||a+2<s?u=r[(a+2)%s]:(El.subVectors(r[s-1],r[s-2]).add(r[s-1]),u=El),this.curveType==="centripetal"||this.curveType==="chordal"){const p=this.curveType==="chordal"?.5:.25;let g=Math.pow(c.distanceToSquared(d),p),y=Math.pow(d.distanceToSquared(f),p),m=Math.pow(f.distanceToSquared(u),p);y<1e-4&&(y=1),g<1e-4&&(g=y),m<1e-4&&(m=y),Zu.initNonuniformCatmullRom(c.x,d.x,f.x,u.x,g,y,m),Qu.initNonuniformCatmullRom(c.y,d.y,f.y,u.y,g,y,m),Ju.initNonuniformCatmullRom(c.z,d.z,f.z,u.z,g,y,m)}else this.curveType==="catmullrom"&&(Zu.initCatmullRom(c.x,d.x,f.x,u.x,this.tension),Qu.initCatmullRom(c.y,d.y,f.y,u.y,this.tension),Ju.initCatmullRom(c.z,d.z,f.z,u.z,this.tension));return i.set(Zu.calc(l),Qu.calc(l),Ju.calc(l)),i}copy(e){super.copy(e),this.points=[];for(let t=0,i=e.points.length;t<i;t++){const r=e.points[t];this.points.push(r.clone())}return this.closed=e.closed,this.curveType=e.curveType,this.tension=e.tension,this}toJSON(){const e=super.toJSON();e.points=[];for(let t=0,i=this.points.length;t<i;t++){const r=this.points[t];e.points.push(r.toArray())}return e.closed=this.closed,e.curveType=this.curveType,e.tension=this.tension,e}fromJSON(e){super.fromJSON(e),this.points=[];for(let t=0,i=e.points.length;t<i;t++){const r=e.points[t];this.points.push(new L().fromArray(r))}return this.closed=e.closed,this.curveType=e.curveType,this.tension=e.tension,this}}class ep extends hn{constructor(e=1,t=32,i=16,r=0,s=Math.PI*2,o=0,a=Math.PI){super(),this.type="SphereGeometry",this.parameters={radius:e,widthSegments:t,heightSegments:i,phiStart:r,phiLength:s,thetaStart:o,thetaLength:a},t=Math.max(3,Math.floor(t)),i=Math.max(2,Math.floor(i));const l=Math.min(o+a,Math.PI);let c=0;const u=[],d=new L,f=new L,p=[],g=[],y=[],m=[];for(let h=0;h<=i;h++){const _=[],v=h/i;let x=0;h===0&&o===0?x=.5/t:h===i&&l===Math.PI&&(x=-.5/t);for(let R=0;R<=t;R++){const A=R/t;d.x=-e*Math.cos(r+A*s)*Math.sin(o+v*a),d.y=e*Math.cos(o+v*a),d.z=e*Math.sin(r+A*s)*Math.sin(o+v*a),g.push(d.x,d.y,d.z),f.copy(d).normalize(),y.push(f.x,f.y,f.z),m.push(A+x,1-v),_.push(c++)}u.push(_)}for(let h=0;h<i;h++)for(let _=0;_<t;_++){const v=u[h][_+1],x=u[h][_],R=u[h+1][_],A=u[h+1][_+1];(h!==0||o>0)&&p.push(v,x,A),(h!==i-1||l<Math.PI)&&p.push(x,R,A)}this.setIndex(p),this.setAttribute("position",new In(g,3)),this.setAttribute("normal",new In(y,3)),this.setAttribute("uv",new In(m,2))}copy(e){return super.copy(e),this.parameters=Object.assign({},e.parameters),this}static fromJSON(e){return new ep(e.radius,e.widthSegments,e.heightSegments,e.phiStart,e.phiLength,e.thetaStart,e.thetaLength)}}class tp extends qn{constructor(e){super(),this.isMeshStandardMaterial=!0,this.defines={STANDARD:""},this.type="MeshStandardMaterial",this.color=new Pe(16777215),this.roughness=1,this.metalness=0,this.map=null,this.lightMap=null,this.lightMapIntensity=1,this.aoMap=null,this.aoMapIntensity=1,this.emissive=new Pe(0),this.emissiveIntensity=1,this.emissiveMap=null,this.bumpMap=null,this.bumpScale=1,this.normalMap=null,this.normalMapType=Lv,this.normalScale=new we(1,1),this.displacementMap=null,this.displacementScale=1,this.displacementBias=0,this.roughnessMap=null,this.metalnessMap=null,this.alphaMap=null,this.envMap=null,this.envMapRotation=new ai,this.envMapIntensity=1,this.wireframe=!1,this.wireframeLinewidth=1,this.wireframeLinecap="round",this.wireframeLinejoin="round",this.flatShading=!1,this.fog=!0,this.setValues(e)}copy(e){return super.copy(e),this.defines={STANDARD:""},this.color.copy(e.color),this.roughness=e.roughness,this.metalness=e.metalness,this.map=e.map,this.lightMap=e.lightMap,this.lightMapIntensity=e.lightMapIntensity,this.aoMap=e.aoMap,this.aoMapIntensity=e.aoMapIntensity,this.emissive.copy(e.emissive),this.emissiveMap=e.emissiveMap,this.emissiveIntensity=e.emissiveIntensity,this.bumpMap=e.bumpMap,this.bumpScale=e.bumpScale,this.normalMap=e.normalMap,this.normalMapType=e.normalMapType,this.normalScale.copy(e.normalScale),this.displacementMap=e.displacementMap,this.displacementScale=e.displacementScale,this.displacementBias=e.displacementBias,this.roughnessMap=e.roughnessMap,this.metalnessMap=e.metalnessMap,this.alphaMap=e.alphaMap,this.envMap=e.envMap,this.envMapRotation.copy(e.envMapRotation),this.envMapIntensity=e.envMapIntensity,this.wireframe=e.wireframe,this.wireframeLinewidth=e.wireframeLinewidth,this.wireframeLinecap=e.wireframeLinecap,this.wireframeLinejoin=e.wireframeLinejoin,this.flatShading=e.flatShading,this.fog=e.fog,this}}class ci extends tp{constructor(e){super(),this.isMeshPhysicalMaterial=!0,this.defines={STANDARD:"",PHYSICAL:""},this.type="MeshPhysicalMaterial",this.anisotropyRotation=0,this.anisotropyMap=null,this.clearcoatMap=null,this.clearcoatRoughness=0,this.clearcoatRoughnessMap=null,this.clearcoatNormalScale=new we(1,1),this.clearcoatNormalMap=null,this.ior=1.5,Object.defineProperty(this,"reflectivity",{get:function(){return Ut(2.5*(this.ior-1)/(this.ior+1),0,1)},set:function(t){this.ior=(1+.4*t)/(1-.4*t)}}),this.iridescenceMap=null,this.iridescenceIOR=1.3,this.iridescenceThicknessRange=[100,400],this.iridescenceThicknessMap=null,this.sheenColor=new Pe(0),this.sheenColorMap=null,this.sheenRoughness=1,this.sheenRoughnessMap=null,this.transmissionMap=null,this.thickness=0,this.thicknessMap=null,this.attenuationDistance=1/0,this.attenuationColor=new Pe(1,1,1),this.specularIntensity=1,this.specularIntensityMap=null,this.specularColor=new Pe(1,1,1),this.specularColorMap=null,this._anisotropy=0,this._clearcoat=0,this._dispersion=0,this._iridescence=0,this._sheen=0,this._transmission=0,this.setValues(e)}get anisotropy(){return this._anisotropy}set anisotropy(e){this._anisotropy>0!=e>0&&this.version++,this._anisotropy=e}get clearcoat(){return this._clearcoat}set clearcoat(e){this._clearcoat>0!=e>0&&this.version++,this._clearcoat=e}get iridescence(){return this._iridescence}set iridescence(e){this._iridescence>0!=e>0&&this.version++,this._iridescence=e}get dispersion(){return this._dispersion}set dispersion(e){this._dispersion>0!=e>0&&this.version++,this._dispersion=e}get sheen(){return this._sheen}set sheen(e){this._sheen>0!=e>0&&this.version++,this._sheen=e}get transmission(){return this._transmission}set transmission(e){this._transmission>0!=e>0&&this.version++,this._transmission=e}copy(e){return super.copy(e),this.defines={STANDARD:"",PHYSICAL:""},this.anisotropy=e.anisotropy,this.anisotropyRotation=e.anisotropyRotation,this.anisotropyMap=e.anisotropyMap,this.clearcoat=e.clearcoat,this.clearcoatMap=e.clearcoatMap,this.clearcoatRoughness=e.clearcoatRoughness,this.clearcoatRoughnessMap=e.clearcoatRoughnessMap,this.clearcoatNormalMap=e.clearcoatNormalMap,this.clearcoatNormalScale.copy(e.clearcoatNormalScale),this.dispersion=e.dispersion,this.ior=e.ior,this.iridescence=e.iridescence,this.iridescenceMap=e.iridescenceMap,this.iridescenceIOR=e.iridescenceIOR,this.iridescenceThicknessRange=[...e.iridescenceThicknessRange],this.iridescenceThicknessMap=e.iridescenceThicknessMap,this.sheen=e.sheen,this.sheenColor.copy(e.sheenColor),this.sheenColorMap=e.sheenColorMap,this.sheenRoughness=e.sheenRoughness,this.sheenRoughnessMap=e.sheenRoughnessMap,this.transmission=e.transmission,this.transmissionMap=e.transmissionMap,this.thickness=e.thickness,this.thicknessMap=e.thicknessMap,this.attenuationDistance=e.attenuationDistance,this.attenuationColor.copy(e.attenuationColor),this.specularIntensity=e.specularIntensity,this.specularIntensityMap=e.specularIntensityMap,this.specularColor.copy(e.specularColor),this.specularColorMap=e.specularColorMap,this}}function Tl(n,e,t){return!n||!t&&n.constructor===e?n:typeof e.BYTES_PER_ELEMENT=="number"?new e(n):Array.prototype.slice.call(n)}function IR(n){return ArrayBuffer.isView(n)&&!(n instanceof DataView)}function NR(n){function e(r,s){return n[r]-n[s]}const t=n.length,i=new Array(t);for(let r=0;r!==t;++r)i[r]=r;return i.sort(e),i}function zg(n,e,t){const i=n.length,r=new n.constructor(i);for(let s=0,o=0;o!==i;++s){const a=t[s]*e;for(let l=0;l!==e;++l)r[o++]=n[a+l]}return r}function ey(n,e,t,i){let r=1,s=n[0];for(;s!==void 0&&s[i]===void 0;)s=n[r++];if(s===void 0)return;let o=s[i];if(o!==void 0)if(Array.isArray(o))do o=s[i],o!==void 0&&(e.push(s.time),t.push.apply(t,o)),s=n[r++];while(s!==void 0);else if(o.toArray!==void 0)do o=s[i],o!==void 0&&(e.push(s.time),o.toArray(t,t.length)),s=n[r++];while(s!==void 0);else do o=s[i],o!==void 0&&(e.push(s.time),t.push(o)),s=n[r++];while(s!==void 0)}class Ra{constructor(e,t,i,r){this.parameterPositions=e,this._cachedIndex=0,this.resultBuffer=r!==void 0?r:new t.constructor(i),this.sampleValues=t,this.valueSize=i,this.settings=null,this.DefaultSettings_={}}evaluate(e){const t=this.parameterPositions;let i=this._cachedIndex,r=t[i],s=t[i-1];e:{t:{let o;n:{i:if(!(e<r)){for(let a=i+2;;){if(r===void 0){if(e<s)break i;return i=t.length,this._cachedIndex=i,this.copySampleValue_(i-1)}if(i===a)break;if(s=r,r=t[++i],e<r)break t}o=t.length;break n}if(!(e>=s)){const a=t[1];e<a&&(i=2,s=a);for(let l=i-2;;){if(s===void 0)return this._cachedIndex=0,this.copySampleValue_(0);if(i===l)break;if(r=s,s=t[--i-1],e>=s)break t}o=i,i=0;break n}break e}for(;i<o;){const a=i+o>>>1;e<t[a]?o=a:i=a+1}if(r=t[i],s=t[i-1],s===void 0)return this._cachedIndex=0,this.copySampleValue_(0);if(r===void 0)return i=t.length,this._cachedIndex=i,this.copySampleValue_(i-1)}this._cachedIndex=i,this.intervalChanged_(i,s,r)}return this.interpolate_(i,s,e,r)}getSettings_(){return this.settings||this.DefaultSettings_}copySampleValue_(e){const t=this.resultBuffer,i=this.sampleValues,r=this.valueSize,s=e*r;for(let o=0;o!==r;++o)t[o]=i[s+o];return t}interpolate_(){throw new Error("call to abstract method")}intervalChanged_(){}}class DR extends Ra{constructor(e,t,i,r){super(e,t,i,r),this._weightPrev=-0,this._offsetPrev=-0,this._weightNext=-0,this._offsetNext=-0,this.DefaultSettings_={endingStart:Rm,endingEnd:Rm}}intervalChanged_(e,t,i){const r=this.parameterPositions;let s=e-2,o=e+1,a=r[s],l=r[o];if(a===void 0)switch(this.getSettings_().endingStart){case Cm:s=e,a=2*t-i;break;case bm:s=r.length-2,a=t+r[s]-r[s+1];break;default:s=e,a=i}if(l===void 0)switch(this.getSettings_().endingEnd){case Cm:o=e,l=2*i-t;break;case bm:o=1,l=i+r[1]-r[0];break;default:o=e-1,l=t}const c=(i-t)*.5,u=this.valueSize;this._weightPrev=c/(t-a),this._weightNext=c/(l-i),this._offsetPrev=s*u,this._offsetNext=o*u}interpolate_(e,t,i,r){const s=this.resultBuffer,o=this.sampleValues,a=this.valueSize,l=e*a,c=l-a,u=this._offsetPrev,d=this._offsetNext,f=this._weightPrev,p=this._weightNext,g=(i-t)/(r-t),y=g*g,m=y*g,h=-f*m+2*f*y-f*g,_=(1+f)*m+(-1.5-2*f)*y+(-.5+f)*g+1,v=(-1-p)*m+(1.5+p)*y+.5*g,x=p*m-p*y;for(let R=0;R!==a;++R)s[R]=h*o[u+R]+_*o[c+R]+v*o[l+R]+x*o[d+R];return s}}class UR extends Ra{constructor(e,t,i,r){super(e,t,i,r)}interpolate_(e,t,i,r){const s=this.resultBuffer,o=this.sampleValues,a=this.valueSize,l=e*a,c=l-a,u=(i-t)/(r-t),d=1-u;for(let f=0;f!==a;++f)s[f]=o[c+f]*d+o[l+f]*u;return s}}class FR extends Ra{constructor(e,t,i,r){super(e,t,i,r)}interpolate_(e){return this.copySampleValue_(e-1)}}class ui{constructor(e,t,i,r){if(e===void 0)throw new Error("THREE.KeyframeTrack: track name is undefined");if(t===void 0||t.length===0)throw new Error("THREE.KeyframeTrack: no keyframes in track named "+e);this.name=e,this.times=Tl(t,this.TimeBufferType),this.values=Tl(i,this.ValueBufferType),this.setInterpolation(r||this.DefaultInterpolation)}static toJSON(e){const t=e.constructor;let i;if(t.toJSON!==this.toJSON)i=t.toJSON(e);else{i={name:e.name,times:Tl(e.times,Array),values:Tl(e.values,Array)};const r=e.getInterpolation();r!==e.DefaultInterpolation&&(i.interpolation=r)}return i.type=e.ValueTypeName,i}InterpolantFactoryMethodDiscrete(e){return new FR(this.times,this.values,this.getValueSize(),e)}InterpolantFactoryMethodLinear(e){return new UR(this.times,this.values,this.getValueSize(),e)}InterpolantFactoryMethodSmooth(e){return new DR(this.times,this.values,this.getValueSize(),e)}setInterpolation(e){let t;switch(e){case pa:t=this.InterpolantFactoryMethodDiscrete;break;case ma:t=this.InterpolantFactoryMethodLinear;break;case Su:t=this.InterpolantFactoryMethodSmooth;break}if(t===void 0){const i="unsupported interpolation for "+this.ValueTypeName+" keyframe track named "+this.name;if(this.createInterpolant===void 0)if(e!==this.DefaultInterpolation)this.setInterpolation(this.DefaultInterpolation);else throw new Error(i);return console.warn("THREE.KeyframeTrack:",i),this}return this.createInterpolant=t,this}getInterpolation(){switch(this.createInterpolant){case this.InterpolantFactoryMethodDiscrete:return pa;case this.InterpolantFactoryMethodLinear:return ma;case this.InterpolantFactoryMethodSmooth:return Su}}getValueSize(){return this.values.length/this.times.length}shift(e){if(e!==0){const t=this.times;for(let i=0,r=t.length;i!==r;++i)t[i]+=e}return this}scale(e){if(e!==1){const t=this.times;for(let i=0,r=t.length;i!==r;++i)t[i]*=e}return this}trim(e,t){const i=this.times,r=i.length;let s=0,o=r-1;for(;s!==r&&i[s]<e;)++s;for(;o!==-1&&i[o]>t;)--o;if(++o,s!==0||o!==r){s>=o&&(o=Math.max(o,1),s=o-1);const a=this.getValueSize();this.times=i.slice(s,o),this.values=this.values.slice(s*a,o*a)}return this}validate(){let e=!0;const t=this.getValueSize();t-Math.floor(t)!==0&&(console.error("THREE.KeyframeTrack: Invalid value size in track.",this),e=!1);const i=this.times,r=this.values,s=i.length;s===0&&(console.error("THREE.KeyframeTrack: Track is empty.",this),e=!1);let o=null;for(let a=0;a!==s;a++){const l=i[a];if(typeof l=="number"&&isNaN(l)){console.error("THREE.KeyframeTrack: Time is not a valid number.",this,a,l),e=!1;break}if(o!==null&&o>l){console.error("THREE.KeyframeTrack: Out of order keys.",this,a,l,o),e=!1;break}o=l}if(r!==void 0&&IR(r))for(let a=0,l=r.length;a!==l;++a){const c=r[a];if(isNaN(c)){console.error("THREE.KeyframeTrack: Value is not a valid number.",this,a,c),e=!1;break}}return e}optimize(){const e=this.times.slice(),t=this.values.slice(),i=this.getValueSize(),r=this.getInterpolation()===Su,s=e.length-1;let o=1;for(let a=1;a<s;++a){let l=!1;const c=e[a],u=e[a+1];if(c!==u&&(a!==1||c!==e[0]))if(r)l=!0;else{const d=a*i,f=d-i,p=d+i;for(let g=0;g!==i;++g){const y=t[d+g];if(y!==t[f+g]||y!==t[p+g]){l=!0;break}}}if(l){if(a!==o){e[o]=e[a];const d=a*i,f=o*i;for(let p=0;p!==i;++p)t[f+p]=t[d+p]}++o}}if(s>0){e[o]=e[s];for(let a=s*i,l=o*i,c=0;c!==i;++c)t[l+c]=t[a+c];++o}return o!==e.length?(this.times=e.slice(0,o),this.values=t.slice(0,o*i)):(this.times=e,this.values=t),this}clone(){const e=this.times.slice(),t=this.values.slice(),i=this.constructor,r=new i(this.name,e,t);return r.createInterpolant=this.createInterpolant,r}}ui.prototype.TimeBufferType=Float32Array;ui.prototype.ValueBufferType=Float32Array;ui.prototype.DefaultInterpolation=ma;class ao extends ui{constructor(e,t,i){super(e,t,i)}}ao.prototype.ValueTypeName="bool";ao.prototype.ValueBufferType=Array;ao.prototype.DefaultInterpolation=pa;ao.prototype.InterpolantFactoryMethodLinear=void 0;ao.prototype.InterpolantFactoryMethodSmooth=void 0;class ty extends ui{}ty.prototype.ValueTypeName="color";class Js extends ui{}Js.prototype.ValueTypeName="number";class OR extends Ra{constructor(e,t,i,r){super(e,t,i,r)}interpolate_(e,t,i,r){const s=this.resultBuffer,o=this.sampleValues,a=this.valueSize,l=(i-t)/(r-t);let c=e*a;for(let u=c+a;c!==u;c+=4)gr.slerpFlat(s,0,o,c-a,o,c,l);return s}}class eo extends ui{InterpolantFactoryMethodLinear(e){return new OR(this.times,this.values,this.getValueSize(),e)}}eo.prototype.ValueTypeName="quaternion";eo.prototype.InterpolantFactoryMethodSmooth=void 0;class lo extends ui{constructor(e,t,i){super(e,t,i)}}lo.prototype.ValueTypeName="string";lo.prototype.ValueBufferType=Array;lo.prototype.DefaultInterpolation=pa;lo.prototype.InterpolantFactoryMethodLinear=void 0;lo.prototype.InterpolantFactoryMethodSmooth=void 0;class to extends ui{}to.prototype.ValueTypeName="vector";class kR{constructor(e="",t=-1,i=[],r=FM){this.name=e,this.tracks=i,this.duration=t,this.blendMode=r,this.uuid=Kn(),this.duration<0&&this.resetDuration()}static parse(e){const t=[],i=e.tracks,r=1/(e.fps||1);for(let o=0,a=i.length;o!==a;++o)t.push(zR(i[o]).scale(r));const s=new this(e.name,e.duration,t,e.blendMode);return s.uuid=e.uuid,s}static toJSON(e){const t=[],i=e.tracks,r={name:e.name,duration:e.duration,tracks:t,uuid:e.uuid,blendMode:e.blendMode};for(let s=0,o=i.length;s!==o;++s)t.push(ui.toJSON(i[s]));return r}static CreateFromMorphTargetSequence(e,t,i,r){const s=t.length,o=[];for(let a=0;a<s;a++){let l=[],c=[];l.push((a+s-1)%s,a,(a+1)%s),c.push(0,1,0);const u=NR(l);l=zg(l,1,u),c=zg(c,1,u),!r&&l[0]===0&&(l.push(s),c.push(c[0])),o.push(new Js(".morphTargetInfluences["+t[a].name+"]",l,c).scale(1/i))}return new this(e,-1,o)}static findByName(e,t){let i=e;if(!Array.isArray(e)){const r=e;i=r.geometry&&r.geometry.animations||r.animations}for(let r=0;r<i.length;r++)if(i[r].name===t)return i[r];return null}static CreateClipsFromMorphTargetSequences(e,t,i){const r={},s=/^([\w-]*?)([\d]+)$/;for(let a=0,l=e.length;a<l;a++){const c=e[a],u=c.name.match(s);if(u&&u.length>1){const d=u[1];let f=r[d];f||(r[d]=f=[]),f.push(c)}}const o=[];for(const a in r)o.push(this.CreateFromMorphTargetSequence(a,r[a],t,i));return o}static parseAnimation(e,t){if(!e)return console.error("THREE.AnimationClip: No animation in JSONLoader data."),null;const i=function(d,f,p,g,y){if(p.length!==0){const m=[],h=[];ey(p,m,h,g),m.length!==0&&y.push(new d(f,m,h))}},r=[],s=e.name||"default",o=e.fps||30,a=e.blendMode;let l=e.length||-1;const c=e.hierarchy||[];for(let d=0;d<c.length;d++){const f=c[d].keys;if(!(!f||f.length===0))if(f[0].morphTargets){const p={};let g;for(g=0;g<f.length;g++)if(f[g].morphTargets)for(let y=0;y<f[g].morphTargets.length;y++)p[f[g].morphTargets[y]]=-1;for(const y in p){const m=[],h=[];for(let _=0;_!==f[g].morphTargets.length;++_){const v=f[g];m.push(v.time),h.push(v.morphTarget===y?1:0)}r.push(new Js(".morphTargetInfluence["+y+"]",m,h))}l=p.length*o}else{const p=".bones["+t[d].name+"]";i(to,p+".position",f,"pos",r),i(eo,p+".quaternion",f,"rot",r),i(to,p+".scale",f,"scl",r)}}return r.length===0?null:new this(s,l,r,a)}resetDuration(){const e=this.tracks;let t=0;for(let i=0,r=e.length;i!==r;++i){const s=this.tracks[i];t=Math.max(t,s.times[s.times.length-1])}return this.duration=t,this}trim(){for(let e=0;e<this.tracks.length;e++)this.tracks[e].trim(0,this.duration);return this}validate(){let e=!0;for(let t=0;t<this.tracks.length;t++)e=e&&this.tracks[t].validate();return e}optimize(){for(let e=0;e<this.tracks.length;e++)this.tracks[e].optimize();return this}clone(){const e=[];for(let t=0;t<this.tracks.length;t++)e.push(this.tracks[t].clone());return new this.constructor(this.name,this.duration,e,this.blendMode)}toJSON(){return this.constructor.toJSON(this)}}function BR(n){switch(n.toLowerCase()){case"scalar":case"double":case"float":case"number":case"integer":return Js;case"vector":case"vector2":case"vector3":case"vector4":return to;case"color":return ty;case"quaternion":return eo;case"bool":case"boolean":return ao;case"string":return lo}throw new Error("THREE.KeyframeTrack: Unsupported typeName: "+n)}function zR(n){if(n.type===void 0)throw new Error("THREE.KeyframeTrack: track type undefined, can not parse");const e=BR(n.type);if(n.times===void 0){const t=[],i=[];ey(n.keys,t,i,"value"),n.times=t,n.values=i}return e.parse!==void 0?e.parse(n):new e(n.name,n.times,n.values,n.interpolation)}const er={enabled:!1,files:{},add:function(n,e){this.enabled!==!1&&(this.files[n]=e)},get:function(n){if(this.enabled!==!1)return this.files[n]},remove:function(n){delete this.files[n]},clear:function(){this.files={}}};class HR{constructor(e,t,i){const r=this;let s=!1,o=0,a=0,l;const c=[];this.onStart=void 0,this.onLoad=e,this.onProgress=t,this.onError=i,this.itemStart=function(u){a++,s===!1&&r.onStart!==void 0&&r.onStart(u,o,a),s=!0},this.itemEnd=function(u){o++,r.onProgress!==void 0&&r.onProgress(u,o,a),o===a&&(s=!1,r.onLoad!==void 0&&r.onLoad())},this.itemError=function(u){r.onError!==void 0&&r.onError(u)},this.resolveURL=function(u){return l?l(u):u},this.setURLModifier=function(u){return l=u,this},this.addHandler=function(u,d){return c.push(u,d),this},this.removeHandler=function(u){const d=c.indexOf(u);return d!==-1&&c.splice(d,2),this},this.getHandler=function(u){for(let d=0,f=c.length;d<f;d+=2){const p=c[d],g=c[d+1];if(p.global&&(p.lastIndex=0),p.test(u))return g}return null}}}const VR=new HR;class co{constructor(e){this.manager=e!==void 0?e:VR,this.crossOrigin="anonymous",this.withCredentials=!1,this.path="",this.resourcePath="",this.requestHeader={}}load(){}loadAsync(e,t){const i=this;return new Promise(function(r,s){i.load(e,r,t,s)})}parse(){}setCrossOrigin(e){return this.crossOrigin=e,this}setWithCredentials(e){return this.withCredentials=e,this}setPath(e){return this.path=e,this}setResourcePath(e){return this.resourcePath=e,this}setRequestHeader(e){return this.requestHeader=e,this}}co.DEFAULT_MATERIAL_NAME="__DEFAULT";const _i={};class GR extends Error{constructor(e,t){super(e),this.response=t}}class ny extends co{constructor(e){super(e)}load(e,t,i,r){e===void 0&&(e=""),this.path!==void 0&&(e=this.path+e),e=this.manager.resolveURL(e);const s=er.get(e);if(s!==void 0)return this.manager.itemStart(e),setTimeout(()=>{t&&t(s),this.manager.itemEnd(e)},0),s;if(_i[e]!==void 0){_i[e].push({onLoad:t,onProgress:i,onError:r});return}_i[e]=[],_i[e].push({onLoad:t,onProgress:i,onError:r});const o=new Request(e,{headers:new Headers(this.requestHeader),credentials:this.withCredentials?"include":"same-origin"}),a=this.mimeType,l=this.responseType;fetch(o).then(c=>{if(c.status===200||c.status===0){if(c.status===0&&console.warn("THREE.FileLoader: HTTP Status 0 received."),typeof ReadableStream>"u"||c.body===void 0||c.body.getReader===void 0)return c;const u=_i[e],d=c.body.getReader(),f=c.headers.get("X-File-Size")||c.headers.get("Content-Length"),p=f?parseInt(f):0,g=p!==0;let y=0;const m=new ReadableStream({start(h){_();function _(){d.read().then(({done:v,value:x})=>{if(v)h.close();else{y+=x.byteLength;const R=new ProgressEvent("progress",{lengthComputable:g,loaded:y,total:p});for(let A=0,w=u.length;A<w;A++){const b=u[A];b.onProgress&&b.onProgress(R)}h.enqueue(x),_()}},v=>{h.error(v)})}}});return new Response(m)}else throw new GR(`fetch for "${c.url}" responded with ${c.status}: ${c.statusText}`,c)}).then(c=>{switch(l){case"arraybuffer":return c.arrayBuffer();case"blob":return c.blob();case"document":return c.text().then(u=>new DOMParser().parseFromString(u,a));case"json":return c.json();default:if(a===void 0)return c.text();{const d=/charset="?([^;"\s]*)"?/i.exec(a),f=d&&d[1]?d[1].toLowerCase():void 0,p=new TextDecoder(f);return c.arrayBuffer().then(g=>p.decode(g))}}}).then(c=>{er.add(e,c);const u=_i[e];delete _i[e];for(let d=0,f=u.length;d<f;d++){const p=u[d];p.onLoad&&p.onLoad(c)}}).catch(c=>{const u=_i[e];if(u===void 0)throw this.manager.itemError(e),c;delete _i[e];for(let d=0,f=u.length;d<f;d++){const p=u[d];p.onError&&p.onError(c)}this.manager.itemError(e)}).finally(()=>{this.manager.itemEnd(e)}),this.manager.itemStart(e)}setResponseType(e){return this.responseType=e,this}setMimeType(e){return this.mimeType=e,this}}class WR extends co{constructor(e){super(e)}load(e,t,i,r){this.path!==void 0&&(e=this.path+e),e=this.manager.resolveURL(e);const s=this,o=er.get(e);if(o!==void 0)return s.manager.itemStart(e),setTimeout(function(){t&&t(o),s.manager.itemEnd(e)},0),o;const a=ga("img");function l(){u(),er.add(e,this),t&&t(this),s.manager.itemEnd(e)}function c(d){u(),r&&r(d),s.manager.itemError(e),s.manager.itemEnd(e)}function u(){a.removeEventListener("load",l,!1),a.removeEventListener("error",c,!1)}return a.addEventListener("load",l,!1),a.addEventListener("error",c,!1),e.slice(0,5)!=="data:"&&this.crossOrigin!==void 0&&(a.crossOrigin=this.crossOrigin),s.manager.itemStart(e),a.src=e,a}}class XR extends co{constructor(e){super(e)}load(e,t,i,r){const s=new Pt,o=new WR(this.manager);return o.setCrossOrigin(this.crossOrigin),o.setPath(this.path),o.load(e,function(a){s.image=a,s.needsUpdate=!0,t!==void 0&&t(s)},i,r),s}}class np extends dt{constructor(e,t=1){super(),this.isLight=!0,this.type="Light",this.color=new Pe(e),this.intensity=t}dispose(){}copy(e,t){return super.copy(e,t),this.color.copy(e.color),this.intensity=e.intensity,this}toJSON(e){const t=super.toJSON(e);return t.object.color=this.color.getHex(),t.object.intensity=this.intensity,this.groundColor!==void 0&&(t.object.groundColor=this.groundColor.getHex()),this.distance!==void 0&&(t.object.distance=this.distance),this.angle!==void 0&&(t.object.angle=this.angle),this.decay!==void 0&&(t.object.decay=this.decay),this.penumbra!==void 0&&(t.object.penumbra=this.penumbra),this.shadow!==void 0&&(t.object.shadow=this.shadow.toJSON()),this.target!==void 0&&(t.object.target=this.target.uuid),t}}const ed=new Ue,Hg=new L,Vg=new L;class ip{constructor(e){this.camera=e,this.intensity=1,this.bias=0,this.normalBias=0,this.radius=1,this.blurSamples=8,this.mapSize=new we(512,512),this.map=null,this.mapPass=null,this.matrix=new Ue,this.autoUpdate=!0,this.needsUpdate=!1,this._frustum=new Kh,this._frameExtents=new we(1,1),this._viewportCount=1,this._viewports=[new st(0,0,1,1)]}getViewportCount(){return this._viewportCount}getFrustum(){return this._frustum}updateMatrices(e){const t=this.camera,i=this.matrix;Hg.setFromMatrixPosition(e.matrixWorld),t.position.copy(Hg),Vg.setFromMatrixPosition(e.target.matrixWorld),t.lookAt(Vg),t.updateMatrixWorld(),ed.multiplyMatrices(t.projectionMatrix,t.matrixWorldInverse),this._frustum.setFromProjectionMatrix(ed),i.set(.5,0,0,.5,0,.5,0,.5,0,0,.5,.5,0,0,0,1),i.multiply(ed)}getViewport(e){return this._viewports[e]}getFrameExtents(){return this._frameExtents}dispose(){this.map&&this.map.dispose(),this.mapPass&&this.mapPass.dispose()}copy(e){return this.camera=e.camera.clone(),this.intensity=e.intensity,this.bias=e.bias,this.radius=e.radius,this.mapSize.copy(e.mapSize),this}clone(){return new this.constructor().copy(this)}toJSON(){const e={};return this.intensity!==1&&(e.intensity=this.intensity),this.bias!==0&&(e.bias=this.bias),this.normalBias!==0&&(e.normalBias=this.normalBias),this.radius!==1&&(e.radius=this.radius),(this.mapSize.x!==512||this.mapSize.y!==512)&&(e.mapSize=this.mapSize.toArray()),e.camera=this.camera.toJSON(!1).object,delete e.camera.matrix,e}}class jR extends ip{constructor(){super(new Jt(50,1,.5,500)),this.isSpotLightShadow=!0,this.focus=1}updateMatrices(e){const t=this.camera,i=Zs*2*e.angle*this.focus,r=this.mapSize.width/this.mapSize.height,s=e.distance||t.far;(i!==t.fov||r!==t.aspect||s!==t.far)&&(t.fov=i,t.aspect=r,t.far=s,t.updateProjectionMatrix()),super.updateMatrices(e)}copy(e){return super.copy(e),this.focus=e.focus,this}}class YR extends np{constructor(e,t,i=0,r=Math.PI/3,s=0,o=2){super(e,t),this.isSpotLight=!0,this.type="SpotLight",this.position.copy(dt.DEFAULT_UP),this.updateMatrix(),this.target=new dt,this.distance=i,this.angle=r,this.penumbra=s,this.decay=o,this.map=null,this.shadow=new jR}get power(){return this.intensity*Math.PI}set power(e){this.intensity=e/Math.PI}dispose(){this.shadow.dispose()}copy(e,t){return super.copy(e,t),this.distance=e.distance,this.angle=e.angle,this.penumbra=e.penumbra,this.decay=e.decay,this.target=e.target.clone(),this.shadow=e.shadow.clone(),this}}const Gg=new Ue,Po=new L,td=new L;class KR extends ip{constructor(){super(new Jt(90,1,.5,500)),this.isPointLightShadow=!0,this._frameExtents=new we(4,2),this._viewportCount=6,this._viewports=[new st(2,1,1,1),new st(0,1,1,1),new st(3,1,1,1),new st(1,1,1,1),new st(3,0,1,1),new st(1,0,1,1)],this._cubeDirections=[new L(1,0,0),new L(-1,0,0),new L(0,0,1),new L(0,0,-1),new L(0,1,0),new L(0,-1,0)],this._cubeUps=[new L(0,1,0),new L(0,1,0),new L(0,1,0),new L(0,1,0),new L(0,0,1),new L(0,0,-1)]}updateMatrices(e,t=0){const i=this.camera,r=this.matrix,s=e.distance||i.far;s!==i.far&&(i.far=s,i.updateProjectionMatrix()),Po.setFromMatrixPosition(e.matrixWorld),i.position.copy(Po),td.copy(i.position),td.add(this._cubeDirections[t]),i.up.copy(this._cubeUps[t]),i.lookAt(td),i.updateMatrixWorld(),r.makeTranslation(-Po.x,-Po.y,-Po.z),Gg.multiplyMatrices(i.projectionMatrix,i.matrixWorldInverse),this._frustum.setFromProjectionMatrix(Gg)}}class qR extends np{constructor(e,t,i=0,r=2){super(e,t),this.isPointLight=!0,this.type="PointLight",this.distance=i,this.decay=r,this.shadow=new KR}get power(){return this.intensity*4*Math.PI}set power(e){this.intensity=e/(4*Math.PI)}dispose(){this.shadow.dispose()}copy(e,t){return super.copy(e,t),this.distance=e.distance,this.decay=e.decay,this.shadow=e.shadow.clone(),this}}class $R extends ip{constructor(){super(new qh(-5,5,5,-5,.5,500)),this.isDirectionalLightShadow=!0}}class ZR extends np{constructor(e,t){super(e,t),this.isDirectionalLight=!0,this.type="DirectionalLight",this.position.copy(dt.DEFAULT_UP),this.updateMatrix(),this.target=new dt,this.shadow=new $R}dispose(){this.shadow.dispose()}copy(e){return super.copy(e),this.target=e.target.clone(),this.shadow=e.shadow.clone(),this}}class qo{static decodeText(e){if(console.warn("THREE.LoaderUtils: decodeText() has been deprecated with r165 and will be removed with r175. Use TextDecoder instead."),typeof TextDecoder<"u")return new TextDecoder().decode(e);let t="";for(let i=0,r=e.length;i<r;i++)t+=String.fromCharCode(e[i]);try{return decodeURIComponent(escape(t))}catch{return t}}static extractUrlBase(e){const t=e.lastIndexOf("/");return t===-1?"./":e.slice(0,t+1)}static resolveURL(e,t){return typeof e!="string"||e===""?"":(/^https?:\/\//i.test(t)&&/^\//.test(e)&&(t=t.replace(/(^https?:\/\/[^\/]+).*/i,"$1")),/^(https?:)?\/\//i.test(e)||/^data:.*,.*$/i.test(e)||/^blob:.*$/i.test(e)?e:t+e)}}class QR extends co{constructor(e){super(e),this.isImageBitmapLoader=!0,typeof createImageBitmap>"u"&&console.warn("THREE.ImageBitmapLoader: createImageBitmap() not supported."),typeof fetch>"u"&&console.warn("THREE.ImageBitmapLoader: fetch() not supported."),this.options={premultiplyAlpha:"none"}}setOptions(e){return this.options=e,this}load(e,t,i,r){e===void 0&&(e=""),this.path!==void 0&&(e=this.path+e),e=this.manager.resolveURL(e);const s=this,o=er.get(e);if(o!==void 0){if(s.manager.itemStart(e),o.then){o.then(c=>{t&&t(c),s.manager.itemEnd(e)}).catch(c=>{r&&r(c)});return}return setTimeout(function(){t&&t(o),s.manager.itemEnd(e)},0),o}const a={};a.credentials=this.crossOrigin==="anonymous"?"same-origin":"include",a.headers=this.requestHeader;const l=fetch(e,a).then(function(c){return c.blob()}).then(function(c){return createImageBitmap(c,Object.assign(s.options,{colorSpaceConversion:"none"}))}).then(function(c){return er.add(e,c),t&&t(c),s.manager.itemEnd(e),c}).catch(function(c){r&&r(c),er.remove(e),s.manager.itemError(e),s.manager.itemEnd(e)});er.add(e,l),s.manager.itemStart(e)}}class JR{constructor(e=!0){this.autoStart=e,this.startTime=0,this.oldTime=0,this.elapsedTime=0,this.running=!1}start(){this.startTime=Wg(),this.oldTime=this.startTime,this.elapsedTime=0,this.running=!0}stop(){this.getElapsedTime(),this.running=!1,this.autoStart=!1}getElapsedTime(){return this.getDelta(),this.elapsedTime}getDelta(){let e=0;if(this.autoStart&&!this.running)return this.start(),0;if(this.running){const t=Wg();e=(t-this.oldTime)/1e3,this.oldTime=t,this.elapsedTime+=e}return e}}function Wg(){return(typeof performance>"u"?Date:performance).now()}const rp="\\[\\]\\.:\\/",eC=new RegExp("["+rp+"]","g"),sp="[^"+rp+"]",tC="[^"+rp.replace("\\.","")+"]",nC=/((?:WC+[\/:])*)/.source.replace("WC",sp),iC=/(WCOD+)?/.source.replace("WCOD",tC),rC=/(?:\.(WC+)(?:\[(.+)\])?)?/.source.replace("WC",sp),sC=/\.(WC+)(?:\[(.+)\])?/.source.replace("WC",sp),oC=new RegExp("^"+nC+iC+rC+sC+"$"),aC=["material","materials","bones","map"];class lC{constructor(e,t,i){const r=i||nt.parseTrackName(t);this._targetGroup=e,this._bindings=e.subscribe_(t,r)}getValue(e,t){this.bind();const i=this._targetGroup.nCachedObjects_,r=this._bindings[i];r!==void 0&&r.getValue(e,t)}setValue(e,t){const i=this._bindings;for(let r=this._targetGroup.nCachedObjects_,s=i.length;r!==s;++r)i[r].setValue(e,t)}bind(){const e=this._bindings;for(let t=this._targetGroup.nCachedObjects_,i=e.length;t!==i;++t)e[t].bind()}unbind(){const e=this._bindings;for(let t=this._targetGroup.nCachedObjects_,i=e.length;t!==i;++t)e[t].unbind()}}class nt{constructor(e,t,i){this.path=t,this.parsedPath=i||nt.parseTrackName(t),this.node=nt.findNode(e,this.parsedPath.nodeName),this.rootNode=e,this.getValue=this._getValue_unbound,this.setValue=this._setValue_unbound}static create(e,t,i){return e&&e.isAnimationObjectGroup?new nt.Composite(e,t,i):new nt(e,t,i)}static sanitizeNodeName(e){return e.replace(/\s/g,"_").replace(eC,"")}static parseTrackName(e){const t=oC.exec(e);if(t===null)throw new Error("PropertyBinding: Cannot parse trackName: "+e);const i={nodeName:t[2],objectName:t[3],objectIndex:t[4],propertyName:t[5],propertyIndex:t[6]},r=i.nodeName&&i.nodeName.lastIndexOf(".");if(r!==void 0&&r!==-1){const s=i.nodeName.substring(r+1);aC.indexOf(s)!==-1&&(i.nodeName=i.nodeName.substring(0,r),i.objectName=s)}if(i.propertyName===null||i.propertyName.length===0)throw new Error("PropertyBinding: can not parse propertyName from trackName: "+e);return i}static findNode(e,t){if(t===void 0||t===""||t==="."||t===-1||t===e.name||t===e.uuid)return e;if(e.skeleton){const i=e.skeleton.getBoneByName(t);if(i!==void 0)return i}if(e.children){const i=function(s){for(let o=0;o<s.length;o++){const a=s[o];if(a.name===t||a.uuid===t)return a;const l=i(a.children);if(l)return l}return null},r=i(e.children);if(r)return r}return null}_getValue_unavailable(){}_setValue_unavailable(){}_getValue_direct(e,t){e[t]=this.targetObject[this.propertyName]}_getValue_array(e,t){const i=this.resolvedProperty;for(let r=0,s=i.length;r!==s;++r)e[t++]=i[r]}_getValue_arrayElement(e,t){e[t]=this.resolvedProperty[this.propertyIndex]}_getValue_toArray(e,t){this.resolvedProperty.toArray(e,t)}_setValue_direct(e,t){this.targetObject[this.propertyName]=e[t]}_setValue_direct_setNeedsUpdate(e,t){this.targetObject[this.propertyName]=e[t],this.targetObject.needsUpdate=!0}_setValue_direct_setMatrixWorldNeedsUpdate(e,t){this.targetObject[this.propertyName]=e[t],this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_array(e,t){const i=this.resolvedProperty;for(let r=0,s=i.length;r!==s;++r)i[r]=e[t++]}_setValue_array_setNeedsUpdate(e,t){const i=this.resolvedProperty;for(let r=0,s=i.length;r!==s;++r)i[r]=e[t++];this.targetObject.needsUpdate=!0}_setValue_array_setMatrixWorldNeedsUpdate(e,t){const i=this.resolvedProperty;for(let r=0,s=i.length;r!==s;++r)i[r]=e[t++];this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_arrayElement(e,t){this.resolvedProperty[this.propertyIndex]=e[t]}_setValue_arrayElement_setNeedsUpdate(e,t){this.resolvedProperty[this.propertyIndex]=e[t],this.targetObject.needsUpdate=!0}_setValue_arrayElement_setMatrixWorldNeedsUpdate(e,t){this.resolvedProperty[this.propertyIndex]=e[t],this.targetObject.matrixWorldNeedsUpdate=!0}_setValue_fromArray(e,t){this.resolvedProperty.fromArray(e,t)}_setValue_fromArray_setNeedsUpdate(e,t){this.resolvedProperty.fromArray(e,t),this.targetObject.needsUpdate=!0}_setValue_fromArray_setMatrixWorldNeedsUpdate(e,t){this.resolvedProperty.fromArray(e,t),this.targetObject.matrixWorldNeedsUpdate=!0}_getValue_unbound(e,t){this.bind(),this.getValue(e,t)}_setValue_unbound(e,t){this.bind(),this.setValue(e,t)}bind(){let e=this.node;const t=this.parsedPath,i=t.objectName,r=t.propertyName;let s=t.propertyIndex;if(e||(e=nt.findNode(this.rootNode,t.nodeName),this.node=e),this.getValue=this._getValue_unavailable,this.setValue=this._setValue_unavailable,!e){console.warn("THREE.PropertyBinding: No target node found for track: "+this.path+".");return}if(i){let c=t.objectIndex;switch(i){case"materials":if(!e.material){console.error("THREE.PropertyBinding: Can not bind to material as node does not have a material.",this);return}if(!e.material.materials){console.error("THREE.PropertyBinding: Can not bind to material.materials as node.material does not have a materials array.",this);return}e=e.material.materials;break;case"bones":if(!e.skeleton){console.error("THREE.PropertyBinding: Can not bind to bones as node does not have a skeleton.",this);return}e=e.skeleton.bones;for(let u=0;u<e.length;u++)if(e[u].name===c){c=u;break}break;case"map":if("map"in e){e=e.map;break}if(!e.material){console.error("THREE.PropertyBinding: Can not bind to material as node does not have a material.",this);return}if(!e.material.map){console.error("THREE.PropertyBinding: Can not bind to material.map as node.material does not have a map.",this);return}e=e.material.map;break;default:if(e[i]===void 0){console.error("THREE.PropertyBinding: Can not bind to objectName of node undefined.",this);return}e=e[i]}if(c!==void 0){if(e[c]===void 0){console.error("THREE.PropertyBinding: Trying to bind to objectIndex of objectName, but is undefined.",this,e);return}e=e[c]}}const o=e[r];if(o===void 0){const c=t.nodeName;console.error("THREE.PropertyBinding: Trying to update property for track: "+c+"."+r+" but it wasn't found.",e);return}let a=this.Versioning.None;this.targetObject=e,e.needsUpdate!==void 0?a=this.Versioning.NeedsUpdate:e.matrixWorldNeedsUpdate!==void 0&&(a=this.Versioning.MatrixWorldNeedsUpdate);let l=this.BindingType.Direct;if(s!==void 0){if(r==="morphTargetInfluences"){if(!e.geometry){console.error("THREE.PropertyBinding: Can not bind to morphTargetInfluences because node does not have a geometry.",this);return}if(!e.geometry.morphAttributes){console.error("THREE.PropertyBinding: Can not bind to morphTargetInfluences because node does not have a geometry.morphAttributes.",this);return}e.morphTargetDictionary[s]!==void 0&&(s=e.morphTargetDictionary[s])}l=this.BindingType.ArrayElement,this.resolvedProperty=o,this.propertyIndex=s}else o.fromArray!==void 0&&o.toArray!==void 0?(l=this.BindingType.HasFromToArray,this.resolvedProperty=o):Array.isArray(o)?(l=this.BindingType.EntireArray,this.resolvedProperty=o):this.propertyName=r;this.getValue=this.GetterByBindingType[l],this.setValue=this.SetterByBindingTypeAndVersioning[l][a]}unbind(){this.node=null,this.getValue=this._getValue_unbound,this.setValue=this._setValue_unbound}}nt.Composite=lC;nt.prototype.BindingType={Direct:0,EntireArray:1,ArrayElement:2,HasFromToArray:3};nt.prototype.Versioning={None:0,NeedsUpdate:1,MatrixWorldNeedsUpdate:2};nt.prototype.GetterByBindingType=[nt.prototype._getValue_direct,nt.prototype._getValue_array,nt.prototype._getValue_arrayElement,nt.prototype._getValue_toArray];nt.prototype.SetterByBindingTypeAndVersioning=[[nt.prototype._setValue_direct,nt.prototype._setValue_direct_setNeedsUpdate,nt.prototype._setValue_direct_setMatrixWorldNeedsUpdate],[nt.prototype._setValue_array,nt.prototype._setValue_array_setNeedsUpdate,nt.prototype._setValue_array_setMatrixWorldNeedsUpdate],[nt.prototype._setValue_arrayElement,nt.prototype._setValue_arrayElement_setNeedsUpdate,nt.prototype._setValue_arrayElement_setMatrixWorldNeedsUpdate],[nt.prototype._setValue_fromArray,nt.prototype._setValue_fromArray_setNeedsUpdate,nt.prototype._setValue_fromArray_setMatrixWorldNeedsUpdate]];const Xg=new Ue;class cC{constructor(e,t,i=0,r=1/0){this.ray=new wa(e,t),this.near=i,this.far=r,this.camera=null,this.layers=new Yh,this.params={Mesh:{},Line:{threshold:1},LOD:{},Points:{threshold:1},Sprite:{}}}set(e,t){this.ray.set(e,t)}setFromCamera(e,t){t.isPerspectiveCamera?(this.ray.origin.setFromMatrixPosition(t.matrixWorld),this.ray.direction.set(e.x,e.y,.5).unproject(t).sub(this.ray.origin).normalize(),this.camera=t):t.isOrthographicCamera?(this.ray.origin.set(e.x,e.y,(t.near+t.far)/(t.near-t.far)).unproject(t),this.ray.direction.set(0,0,-1).transformDirection(t.matrixWorld),this.camera=t):console.error("THREE.Raycaster: Unsupported camera type: "+t.type)}setFromXRController(e){return Xg.identity().extractRotation(e.matrixWorld),this.ray.origin.setFromMatrixPosition(e.matrixWorld),this.ray.direction.set(0,0,-1).applyMatrix4(Xg),this}intersectObject(e,t=!0,i=[]){return Bf(e,this,i,t),i.sort(jg),i}intersectObjects(e,t=!0,i=[]){for(let r=0,s=e.length;r<s;r++)Bf(e[r],this,i,t);return i.sort(jg),i}}function jg(n,e){return n.distance-e.distance}function Bf(n,e,t,i){let r=!0;if(n.layers.test(e.layers)&&n.raycast(e,t)===!1&&(r=!1),r===!0&&i===!0){const s=n.children;for(let o=0,a=s.length;o<a;o++)Bf(s[o],e,t,!0)}}typeof __THREE_DEVTOOLS__<"u"&&__THREE_DEVTOOLS__.dispatchEvent(new CustomEvent("register",{detail:{revision:Oh}}));typeof window<"u"&&(window.__THREE__?console.warn("WARNING: Multiple instances of Three.js being imported."):window.__THREE__=Oh);function Yg(n,e){if(e===OM)return console.warn("THREE.BufferGeometryUtils.toTrianglesDrawMode(): Geometry already defined as triangles."),n;if(e===Nf||e===Pv){let t=n.getIndex();if(t===null){const o=[],a=n.getAttribute("position");if(a!==void 0){for(let l=0;l<a.count;l++)o.push(l);n.setIndex(o),t=n.getIndex()}else return console.error("THREE.BufferGeometryUtils.toTrianglesDrawMode(): Undefined position attribute. Processing not possible."),n}const i=t.count-2,r=[];if(e===Nf)for(let o=1;o<=i;o++)r.push(t.getX(0)),r.push(t.getX(o)),r.push(t.getX(o+1));else for(let o=0;o<i;o++)o%2===0?(r.push(t.getX(o)),r.push(t.getX(o+1)),r.push(t.getX(o+2))):(r.push(t.getX(o+2)),r.push(t.getX(o+1)),r.push(t.getX(o)));r.length/3!==i&&console.error("THREE.BufferGeometryUtils.toTrianglesDrawMode(): Unable to generate correct amount of triangles.");const s=n.clone();return s.setIndex(r),s.clearGroups(),s}else return console.error("THREE.BufferGeometryUtils.toTrianglesDrawMode(): Unknown draw mode:",e),n}class uC extends co{constructor(e){super(e),this.dracoLoader=null,this.ktx2Loader=null,this.meshoptDecoder=null,this.pluginCallbacks=[],this.register(function(t){return new mC(t)}),this.register(function(t){return new gC(t)}),this.register(function(t){return new wC(t)}),this.register(function(t){return new AC(t)}),this.register(function(t){return new RC(t)}),this.register(function(t){return new vC(t)}),this.register(function(t){return new yC(t)}),this.register(function(t){return new xC(t)}),this.register(function(t){return new SC(t)}),this.register(function(t){return new pC(t)}),this.register(function(t){return new MC(t)}),this.register(function(t){return new _C(t)}),this.register(function(t){return new TC(t)}),this.register(function(t){return new EC(t)}),this.register(function(t){return new fC(t)}),this.register(function(t){return new CC(t)}),this.register(function(t){return new bC(t)})}load(e,t,i,r){const s=this;let o;if(this.resourcePath!=="")o=this.resourcePath;else if(this.path!==""){const c=qo.extractUrlBase(e);o=qo.resolveURL(c,this.path)}else o=qo.extractUrlBase(e);this.manager.itemStart(e);const a=function(c){r?r(c):console.error(c),s.manager.itemError(e),s.manager.itemEnd(e)},l=new ny(this.manager);l.setPath(this.path),l.setResponseType("arraybuffer"),l.setRequestHeader(this.requestHeader),l.setWithCredentials(this.withCredentials),l.load(e,function(c){try{s.parse(c,o,function(u){t(u),s.manager.itemEnd(e)},a)}catch(u){a(u)}},i,a)}setDRACOLoader(e){return this.dracoLoader=e,this}setDDSLoader(){throw new Error('THREE.GLTFLoader: "MSFT_texture_dds" no longer supported. Please update to "KHR_texture_basisu".')}setKTX2Loader(e){return this.ktx2Loader=e,this}setMeshoptDecoder(e){return this.meshoptDecoder=e,this}register(e){return this.pluginCallbacks.indexOf(e)===-1&&this.pluginCallbacks.push(e),this}unregister(e){return this.pluginCallbacks.indexOf(e)!==-1&&this.pluginCallbacks.splice(this.pluginCallbacks.indexOf(e),1),this}parse(e,t,i,r){let s;const o={},a={},l=new TextDecoder;if(typeof e=="string")s=JSON.parse(e);else if(e instanceof ArrayBuffer)if(l.decode(new Uint8Array(e,0,4))===iy){try{o[Ve.KHR_BINARY_GLTF]=new PC(e)}catch(d){r&&r(d);return}s=JSON.parse(o[Ve.KHR_BINARY_GLTF].content)}else s=JSON.parse(l.decode(e));else s=e;if(s.asset===void 0||s.asset.version[0]<2){r&&r(new Error("THREE.GLTFLoader: Unsupported asset. glTF versions >=2.0 are supported."));return}const c=new GC(s,{path:t||this.resourcePath||"",crossOrigin:this.crossOrigin,requestHeader:this.requestHeader,manager:this.manager,ktx2Loader:this.ktx2Loader,meshoptDecoder:this.meshoptDecoder});c.fileLoader.setRequestHeader(this.requestHeader);for(let u=0;u<this.pluginCallbacks.length;u++){const d=this.pluginCallbacks[u](c);d.name||console.error("THREE.GLTFLoader: Invalid plugin found: missing name"),a[d.name]=d,o[d.name]=!0}if(s.extensionsUsed)for(let u=0;u<s.extensionsUsed.length;++u){const d=s.extensionsUsed[u],f=s.extensionsRequired||[];switch(d){case Ve.KHR_MATERIALS_UNLIT:o[d]=new hC;break;case Ve.KHR_DRACO_MESH_COMPRESSION:o[d]=new LC(s,this.dracoLoader);break;case Ve.KHR_TEXTURE_TRANSFORM:o[d]=new IC;break;case Ve.KHR_MESH_QUANTIZATION:o[d]=new NC;break;default:f.indexOf(d)>=0&&a[d]===void 0&&console.warn('THREE.GLTFLoader: Unknown extension "'+d+'".')}}c.setExtensions(o),c.setPlugins(a),c.parse(i,r)}parseAsync(e,t){const i=this;return new Promise(function(r,s){i.parse(e,t,r,s)})}}function dC(){let n={};return{get:function(e){return n[e]},add:function(e,t){n[e]=t},remove:function(e){delete n[e]},removeAll:function(){n={}}}}const Ve={KHR_BINARY_GLTF:"KHR_binary_glTF",KHR_DRACO_MESH_COMPRESSION:"KHR_draco_mesh_compression",KHR_LIGHTS_PUNCTUAL:"KHR_lights_punctual",KHR_MATERIALS_CLEARCOAT:"KHR_materials_clearcoat",KHR_MATERIALS_DISPERSION:"KHR_materials_dispersion",KHR_MATERIALS_IOR:"KHR_materials_ior",KHR_MATERIALS_SHEEN:"KHR_materials_sheen",KHR_MATERIALS_SPECULAR:"KHR_materials_specular",KHR_MATERIALS_TRANSMISSION:"KHR_materials_transmission",KHR_MATERIALS_IRIDESCENCE:"KHR_materials_iridescence",KHR_MATERIALS_ANISOTROPY:"KHR_materials_anisotropy",KHR_MATERIALS_UNLIT:"KHR_materials_unlit",KHR_MATERIALS_VOLUME:"KHR_materials_volume",KHR_TEXTURE_BASISU:"KHR_texture_basisu",KHR_TEXTURE_TRANSFORM:"KHR_texture_transform",KHR_MESH_QUANTIZATION:"KHR_mesh_quantization",KHR_MATERIALS_EMISSIVE_STRENGTH:"KHR_materials_emissive_strength",EXT_MATERIALS_BUMP:"EXT_materials_bump",EXT_TEXTURE_WEBP:"EXT_texture_webp",EXT_TEXTURE_AVIF:"EXT_texture_avif",EXT_MESHOPT_COMPRESSION:"EXT_meshopt_compression",EXT_MESH_GPU_INSTANCING:"EXT_mesh_gpu_instancing"};class fC{constructor(e){this.parser=e,this.name=Ve.KHR_LIGHTS_PUNCTUAL,this.cache={refs:{},uses:{}}}_markDefs(){const e=this.parser,t=this.parser.json.nodes||[];for(let i=0,r=t.length;i<r;i++){const s=t[i];s.extensions&&s.extensions[this.name]&&s.extensions[this.name].light!==void 0&&e._addNodeRef(this.cache,s.extensions[this.name].light)}}_loadLight(e){const t=this.parser,i="light:"+e;let r=t.cache.get(i);if(r)return r;const s=t.json,l=((s.extensions&&s.extensions[this.name]||{}).lights||[])[e];let c;const u=new Pe(16777215);l.color!==void 0&&u.setRGB(l.color[0],l.color[1],l.color[2],Vt);const d=l.range!==void 0?l.range:0;switch(l.type){case"directional":c=new ZR(u),c.target.position.set(0,0,-1),c.add(c.target);break;case"point":c=new qR(u),c.distance=d;break;case"spot":c=new YR(u),c.distance=d,l.spot=l.spot||{},l.spot.innerConeAngle=l.spot.innerConeAngle!==void 0?l.spot.innerConeAngle:0,l.spot.outerConeAngle=l.spot.outerConeAngle!==void 0?l.spot.outerConeAngle:Math.PI/4,c.angle=l.spot.outerConeAngle,c.penumbra=1-l.spot.innerConeAngle/l.spot.outerConeAngle,c.target.position.set(0,0,-1),c.add(c.target);break;default:throw new Error("THREE.GLTFLoader: Unexpected light type: "+l.type)}return c.position.set(0,0,0),c.decay=2,xi(c,l),l.intensity!==void 0&&(c.intensity=l.intensity),c.name=t.createUniqueName(l.name||"light_"+e),r=Promise.resolve(c),t.cache.add(i,r),r}getDependency(e,t){if(e==="light")return this._loadLight(t)}createNodeAttachment(e){const t=this,i=this.parser,s=i.json.nodes[e],a=(s.extensions&&s.extensions[this.name]||{}).light;return a===void 0?null:this._loadLight(a).then(function(l){return i._getNodeRef(t.cache,a,l)})}}class hC{constructor(){this.name=Ve.KHR_MATERIALS_UNLIT}getMaterialType(){return Ai}extendParams(e,t,i){const r=[];e.color=new Pe(1,1,1),e.opacity=1;const s=t.pbrMetallicRoughness;if(s){if(Array.isArray(s.baseColorFactor)){const o=s.baseColorFactor;e.color.setRGB(o[0],o[1],o[2],Vt),e.opacity=o[3]}s.baseColorTexture!==void 0&&r.push(i.assignTexture(e,"map",s.baseColorTexture,Qt))}return Promise.all(r)}}class pC{constructor(e){this.parser=e,this.name=Ve.KHR_MATERIALS_EMISSIVE_STRENGTH}extendMaterialParams(e,t){const r=this.parser.json.materials[e];if(!r.extensions||!r.extensions[this.name])return Promise.resolve();const s=r.extensions[this.name].emissiveStrength;return s!==void 0&&(t.emissiveIntensity=s),Promise.resolve()}}class mC{constructor(e){this.parser=e,this.name=Ve.KHR_MATERIALS_CLEARCOAT}getMaterialType(e){const i=this.parser.json.materials[e];return!i.extensions||!i.extensions[this.name]?null:ci}extendMaterialParams(e,t){const i=this.parser,r=i.json.materials[e];if(!r.extensions||!r.extensions[this.name])return Promise.resolve();const s=[],o=r.extensions[this.name];if(o.clearcoatFactor!==void 0&&(t.clearcoat=o.clearcoatFactor),o.clearcoatTexture!==void 0&&s.push(i.assignTexture(t,"clearcoatMap",o.clearcoatTexture)),o.clearcoatRoughnessFactor!==void 0&&(t.clearcoatRoughness=o.clearcoatRoughnessFactor),o.clearcoatRoughnessTexture!==void 0&&s.push(i.assignTexture(t,"clearcoatRoughnessMap",o.clearcoatRoughnessTexture)),o.clearcoatNormalTexture!==void 0&&(s.push(i.assignTexture(t,"clearcoatNormalMap",o.clearcoatNormalTexture)),o.clearcoatNormalTexture.scale!==void 0)){const a=o.clearcoatNormalTexture.scale;t.clearcoatNormalScale=new we(a,a)}return Promise.all(s)}}class gC{constructor(e){this.parser=e,this.name=Ve.KHR_MATERIALS_DISPERSION}getMaterialType(e){const i=this.parser.json.materials[e];return!i.extensions||!i.extensions[this.name]?null:ci}extendMaterialParams(e,t){const r=this.parser.json.materials[e];if(!r.extensions||!r.extensions[this.name])return Promise.resolve();const s=r.extensions[this.name];return t.dispersion=s.dispersion!==void 0?s.dispersion:0,Promise.resolve()}}class _C{constructor(e){this.parser=e,this.name=Ve.KHR_MATERIALS_IRIDESCENCE}getMaterialType(e){const i=this.parser.json.materials[e];return!i.extensions||!i.extensions[this.name]?null:ci}extendMaterialParams(e,t){const i=this.parser,r=i.json.materials[e];if(!r.extensions||!r.extensions[this.name])return Promise.resolve();const s=[],o=r.extensions[this.name];return o.iridescenceFactor!==void 0&&(t.iridescence=o.iridescenceFactor),o.iridescenceTexture!==void 0&&s.push(i.assignTexture(t,"iridescenceMap",o.iridescenceTexture)),o.iridescenceIor!==void 0&&(t.iridescenceIOR=o.iridescenceIor),t.iridescenceThicknessRange===void 0&&(t.iridescenceThicknessRange=[100,400]),o.iridescenceThicknessMinimum!==void 0&&(t.iridescenceThicknessRange[0]=o.iridescenceThicknessMinimum),o.iridescenceThicknessMaximum!==void 0&&(t.iridescenceThicknessRange[1]=o.iridescenceThicknessMaximum),o.iridescenceThicknessTexture!==void 0&&s.push(i.assignTexture(t,"iridescenceThicknessMap",o.iridescenceThicknessTexture)),Promise.all(s)}}class vC{constructor(e){this.parser=e,this.name=Ve.KHR_MATERIALS_SHEEN}getMaterialType(e){const i=this.parser.json.materials[e];return!i.extensions||!i.extensions[this.name]?null:ci}extendMaterialParams(e,t){const i=this.parser,r=i.json.materials[e];if(!r.extensions||!r.extensions[this.name])return Promise.resolve();const s=[];t.sheenColor=new Pe(0,0,0),t.sheenRoughness=0,t.sheen=1;const o=r.extensions[this.name];if(o.sheenColorFactor!==void 0){const a=o.sheenColorFactor;t.sheenColor.setRGB(a[0],a[1],a[2],Vt)}return o.sheenRoughnessFactor!==void 0&&(t.sheenRoughness=o.sheenRoughnessFactor),o.sheenColorTexture!==void 0&&s.push(i.assignTexture(t,"sheenColorMap",o.sheenColorTexture,Qt)),o.sheenRoughnessTexture!==void 0&&s.push(i.assignTexture(t,"sheenRoughnessMap",o.sheenRoughnessTexture)),Promise.all(s)}}class yC{constructor(e){this.parser=e,this.name=Ve.KHR_MATERIALS_TRANSMISSION}getMaterialType(e){const i=this.parser.json.materials[e];return!i.extensions||!i.extensions[this.name]?null:ci}extendMaterialParams(e,t){const i=this.parser,r=i.json.materials[e];if(!r.extensions||!r.extensions[this.name])return Promise.resolve();const s=[],o=r.extensions[this.name];return o.transmissionFactor!==void 0&&(t.transmission=o.transmissionFactor),o.transmissionTexture!==void 0&&s.push(i.assignTexture(t,"transmissionMap",o.transmissionTexture)),Promise.all(s)}}class xC{constructor(e){this.parser=e,this.name=Ve.KHR_MATERIALS_VOLUME}getMaterialType(e){const i=this.parser.json.materials[e];return!i.extensions||!i.extensions[this.name]?null:ci}extendMaterialParams(e,t){const i=this.parser,r=i.json.materials[e];if(!r.extensions||!r.extensions[this.name])return Promise.resolve();const s=[],o=r.extensions[this.name];t.thickness=o.thicknessFactor!==void 0?o.thicknessFactor:0,o.thicknessTexture!==void 0&&s.push(i.assignTexture(t,"thicknessMap",o.thicknessTexture)),t.attenuationDistance=o.attenuationDistance||1/0;const a=o.attenuationColor||[1,1,1];return t.attenuationColor=new Pe().setRGB(a[0],a[1],a[2],Vt),Promise.all(s)}}class SC{constructor(e){this.parser=e,this.name=Ve.KHR_MATERIALS_IOR}getMaterialType(e){const i=this.parser.json.materials[e];return!i.extensions||!i.extensions[this.name]?null:ci}extendMaterialParams(e,t){const r=this.parser.json.materials[e];if(!r.extensions||!r.extensions[this.name])return Promise.resolve();const s=r.extensions[this.name];return t.ior=s.ior!==void 0?s.ior:1.5,Promise.resolve()}}class MC{constructor(e){this.parser=e,this.name=Ve.KHR_MATERIALS_SPECULAR}getMaterialType(e){const i=this.parser.json.materials[e];return!i.extensions||!i.extensions[this.name]?null:ci}extendMaterialParams(e,t){const i=this.parser,r=i.json.materials[e];if(!r.extensions||!r.extensions[this.name])return Promise.resolve();const s=[],o=r.extensions[this.name];t.specularIntensity=o.specularFactor!==void 0?o.specularFactor:1,o.specularTexture!==void 0&&s.push(i.assignTexture(t,"specularIntensityMap",o.specularTexture));const a=o.specularColorFactor||[1,1,1];return t.specularColor=new Pe().setRGB(a[0],a[1],a[2],Vt),o.specularColorTexture!==void 0&&s.push(i.assignTexture(t,"specularColorMap",o.specularColorTexture,Qt)),Promise.all(s)}}class EC{constructor(e){this.parser=e,this.name=Ve.EXT_MATERIALS_BUMP}getMaterialType(e){const i=this.parser.json.materials[e];return!i.extensions||!i.extensions[this.name]?null:ci}extendMaterialParams(e,t){const i=this.parser,r=i.json.materials[e];if(!r.extensions||!r.extensions[this.name])return Promise.resolve();const s=[],o=r.extensions[this.name];return t.bumpScale=o.bumpFactor!==void 0?o.bumpFactor:1,o.bumpTexture!==void 0&&s.push(i.assignTexture(t,"bumpMap",o.bumpTexture)),Promise.all(s)}}class TC{constructor(e){this.parser=e,this.name=Ve.KHR_MATERIALS_ANISOTROPY}getMaterialType(e){const i=this.parser.json.materials[e];return!i.extensions||!i.extensions[this.name]?null:ci}extendMaterialParams(e,t){const i=this.parser,r=i.json.materials[e];if(!r.extensions||!r.extensions[this.name])return Promise.resolve();const s=[],o=r.extensions[this.name];return o.anisotropyStrength!==void 0&&(t.anisotropy=o.anisotropyStrength),o.anisotropyRotation!==void 0&&(t.anisotropyRotation=o.anisotropyRotation),o.anisotropyTexture!==void 0&&s.push(i.assignTexture(t,"anisotropyMap",o.anisotropyTexture)),Promise.all(s)}}class wC{constructor(e){this.parser=e,this.name=Ve.KHR_TEXTURE_BASISU}loadTexture(e){const t=this.parser,i=t.json,r=i.textures[e];if(!r.extensions||!r.extensions[this.name])return null;const s=r.extensions[this.name],o=t.options.ktx2Loader;if(!o){if(i.extensionsRequired&&i.extensionsRequired.indexOf(this.name)>=0)throw new Error("THREE.GLTFLoader: setKTX2Loader must be called before loading KTX2 textures");return null}return t.loadTextureImage(e,s.source,o)}}class AC{constructor(e){this.parser=e,this.name=Ve.EXT_TEXTURE_WEBP,this.isSupported=null}loadTexture(e){const t=this.name,i=this.parser,r=i.json,s=r.textures[e];if(!s.extensions||!s.extensions[t])return null;const o=s.extensions[t],a=r.images[o.source];let l=i.textureLoader;if(a.uri){const c=i.options.manager.getHandler(a.uri);c!==null&&(l=c)}return this.detectSupport().then(function(c){if(c)return i.loadTextureImage(e,o.source,l);if(r.extensionsRequired&&r.extensionsRequired.indexOf(t)>=0)throw new Error("THREE.GLTFLoader: WebP required by asset but unsupported.");return i.loadTexture(e)})}detectSupport(){return this.isSupported||(this.isSupported=new Promise(function(e){const t=new Image;t.src="data:image/webp;base64,UklGRiIAAABXRUJQVlA4IBYAAAAwAQCdASoBAAEADsD+JaQAA3AAAAAA",t.onload=t.onerror=function(){e(t.height===1)}})),this.isSupported}}class RC{constructor(e){this.parser=e,this.name=Ve.EXT_TEXTURE_AVIF,this.isSupported=null}loadTexture(e){const t=this.name,i=this.parser,r=i.json,s=r.textures[e];if(!s.extensions||!s.extensions[t])return null;const o=s.extensions[t],a=r.images[o.source];let l=i.textureLoader;if(a.uri){const c=i.options.manager.getHandler(a.uri);c!==null&&(l=c)}return this.detectSupport().then(function(c){if(c)return i.loadTextureImage(e,o.source,l);if(r.extensionsRequired&&r.extensionsRequired.indexOf(t)>=0)throw new Error("THREE.GLTFLoader: AVIF required by asset but unsupported.");return i.loadTexture(e)})}detectSupport(){return this.isSupported||(this.isSupported=new Promise(function(e){const t=new Image;t.src="data:image/avif;base64,AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADybWV0YQAAAAAAAAAoaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAGxpYmF2aWYAAAAADnBpdG0AAAAAAAEAAAAeaWxvYwAAAABEAAABAAEAAAABAAABGgAAABcAAAAoaWluZgAAAAAAAQAAABppbmZlAgAAAAABAABhdjAxQ29sb3IAAAAAamlwcnAAAABLaXBjbwAAABRpc3BlAAAAAAAAAAEAAAABAAAAEHBpeGkAAAAAAwgICAAAAAxhdjFDgQAMAAAAABNjb2xybmNseAACAAIABoAAAAAXaXBtYQAAAAAAAAABAAEEAQKDBAAAAB9tZGF0EgAKCBgABogQEDQgMgkQAAAAB8dSLfI=",t.onload=t.onerror=function(){e(t.height===1)}})),this.isSupported}}class CC{constructor(e){this.name=Ve.EXT_MESHOPT_COMPRESSION,this.parser=e}loadBufferView(e){const t=this.parser.json,i=t.bufferViews[e];if(i.extensions&&i.extensions[this.name]){const r=i.extensions[this.name],s=this.parser.getDependency("buffer",r.buffer),o=this.parser.options.meshoptDecoder;if(!o||!o.supported){if(t.extensionsRequired&&t.extensionsRequired.indexOf(this.name)>=0)throw new Error("THREE.GLTFLoader: setMeshoptDecoder must be called before loading compressed files");return null}return s.then(function(a){const l=r.byteOffset||0,c=r.byteLength||0,u=r.count,d=r.byteStride,f=new Uint8Array(a,l,c);return o.decodeGltfBufferAsync?o.decodeGltfBufferAsync(u,d,f,r.mode,r.filter).then(function(p){return p.buffer}):o.ready.then(function(){const p=new ArrayBuffer(u*d);return o.decodeGltfBuffer(new Uint8Array(p),u,d,f,r.mode,r.filter),p})})}else return null}}class bC{constructor(e){this.name=Ve.EXT_MESH_GPU_INSTANCING,this.parser=e}createNodeMesh(e){const t=this.parser.json,i=t.nodes[e];if(!i.extensions||!i.extensions[this.name]||i.mesh===void 0)return null;const r=t.meshes[i.mesh];for(const c of r.primitives)if(c.mode!==An.TRIANGLES&&c.mode!==An.TRIANGLE_STRIP&&c.mode!==An.TRIANGLE_FAN&&c.mode!==void 0)return null;const o=i.extensions[this.name].attributes,a=[],l={};for(const c in o)a.push(this.parser.getDependency("accessor",o[c]).then(u=>(l[c]=u,l[c])));return a.length<1?null:(a.push(this.parser.createNodeMesh(e)),Promise.all(a).then(c=>{const u=c.pop(),d=u.isGroup?u.children:[u],f=c[0].count,p=[];for(const g of d){const y=new Ue,m=new L,h=new gr,_=new L(1,1,1),v=new RR(g.geometry,g.material,f);for(let x=0;x<f;x++)l.TRANSLATION&&m.fromBufferAttribute(l.TRANSLATION,x),l.ROTATION&&h.fromBufferAttribute(l.ROTATION,x),l.SCALE&&_.fromBufferAttribute(l.SCALE,x),v.setMatrixAt(x,y.compose(m,h,_));for(const x in l)if(x==="_COLOR_0"){const R=l[x];v.instanceColor=new Of(R.array,R.itemSize,R.normalized)}else x!=="TRANSLATION"&&x!=="ROTATION"&&x!=="SCALE"&&g.geometry.setAttribute(x,l[x]);dt.prototype.copy.call(v,g),this.parser.assignFinalMaterial(v),p.push(v)}return u.isGroup?(u.clear(),u.add(...p),u):p[0]}))}}const iy="glTF",Lo=12,Kg={JSON:1313821514,BIN:5130562};class PC{constructor(e){this.name=Ve.KHR_BINARY_GLTF,this.content=null,this.body=null;const t=new DataView(e,0,Lo),i=new TextDecoder;if(this.header={magic:i.decode(new Uint8Array(e.slice(0,4))),version:t.getUint32(4,!0),length:t.getUint32(8,!0)},this.header.magic!==iy)throw new Error("THREE.GLTFLoader: Unsupported glTF-Binary header.");if(this.header.version<2)throw new Error("THREE.GLTFLoader: Legacy binary file detected.");const r=this.header.length-Lo,s=new DataView(e,Lo);let o=0;for(;o<r;){const a=s.getUint32(o,!0);o+=4;const l=s.getUint32(o,!0);if(o+=4,l===Kg.JSON){const c=new Uint8Array(e,Lo+o,a);this.content=i.decode(c)}else if(l===Kg.BIN){const c=Lo+o;this.body=e.slice(c,c+a)}o+=a}if(this.content===null)throw new Error("THREE.GLTFLoader: JSON content not found.")}}class LC{constructor(e,t){if(!t)throw new Error("THREE.GLTFLoader: No DRACOLoader instance provided.");this.name=Ve.KHR_DRACO_MESH_COMPRESSION,this.json=e,this.dracoLoader=t,this.dracoLoader.preload()}decodePrimitive(e,t){const i=this.json,r=this.dracoLoader,s=e.extensions[this.name].bufferView,o=e.extensions[this.name].attributes,a={},l={},c={};for(const u in o){const d=zf[u]||u.toLowerCase();a[d]=o[u]}for(const u in e.attributes){const d=zf[u]||u.toLowerCase();if(o[u]!==void 0){const f=i.accessors[e.attributes[u]],p=ks[f.componentType];c[d]=p.name,l[d]=f.normalized===!0}}return t.getDependency("bufferView",s).then(function(u){return new Promise(function(d,f){r.decodeDracoFile(u,function(p){for(const g in p.attributes){const y=p.attributes[g],m=l[g];m!==void 0&&(y.normalized=m)}d(p)},a,c,Vt,f)})})}}class IC{constructor(){this.name=Ve.KHR_TEXTURE_TRANSFORM}extendTexture(e,t){return(t.texCoord===void 0||t.texCoord===e.channel)&&t.offset===void 0&&t.rotation===void 0&&t.scale===void 0||(e=e.clone(),t.texCoord!==void 0&&(e.channel=t.texCoord),t.offset!==void 0&&e.offset.fromArray(t.offset),t.rotation!==void 0&&(e.rotation=t.rotation),t.scale!==void 0&&e.repeat.fromArray(t.scale),e.needsUpdate=!0),e}}class NC{constructor(){this.name=Ve.KHR_MESH_QUANTIZATION}}class ry extends Ra{constructor(e,t,i,r){super(e,t,i,r)}copySampleValue_(e){const t=this.resultBuffer,i=this.sampleValues,r=this.valueSize,s=e*r*3+r;for(let o=0;o!==r;o++)t[o]=i[s+o];return t}interpolate_(e,t,i,r){const s=this.resultBuffer,o=this.sampleValues,a=this.valueSize,l=a*2,c=a*3,u=r-t,d=(i-t)/u,f=d*d,p=f*d,g=e*c,y=g-c,m=-2*p+3*f,h=p-f,_=1-m,v=h-f+d;for(let x=0;x!==a;x++){const R=o[y+x+a],A=o[y+x+l]*u,w=o[g+x+a],b=o[g+x]*u;s[x]=_*R+v*A+m*w+h*b}return s}}const DC=new gr;class UC extends ry{interpolate_(e,t,i,r){const s=super.interpolate_(e,t,i,r);return DC.fromArray(s).normalize().toArray(s),s}}const An={POINTS:0,LINES:1,LINE_LOOP:2,LINE_STRIP:3,TRIANGLES:4,TRIANGLE_STRIP:5,TRIANGLE_FAN:6},ks={5120:Int8Array,5121:Uint8Array,5122:Int16Array,5123:Uint16Array,5125:Uint32Array,5126:Float32Array},qg={9728:en,9729:an,9984:xv,9985:kl,9986:Fo,9987:Ti},$g={33071:Ji,33648:_c,10497:Ks},nd={SCALAR:1,VEC2:2,VEC3:3,VEC4:4,MAT2:4,MAT3:9,MAT4:16},zf={POSITION:"position",NORMAL:"normal",TANGENT:"tangent",TEXCOORD_0:"uv",TEXCOORD_1:"uv1",TEXCOORD_2:"uv2",TEXCOORD_3:"uv3",COLOR_0:"color",WEIGHTS_0:"skinWeight",JOINTS_0:"skinIndex"},Xi={scale:"scale",translation:"position",rotation:"quaternion",weights:"morphTargetInfluences"},FC={CUBICSPLINE:void 0,LINEAR:ma,STEP:pa},id={OPAQUE:"OPAQUE",MASK:"MASK",BLEND:"BLEND"};function OC(n){return n.DefaultMaterial===void 0&&(n.DefaultMaterial=new tp({color:16777215,emissive:0,metalness:1,roughness:1,transparent:!1,depthTest:!0,side:Ii})),n.DefaultMaterial}function Tr(n,e,t){for(const i in t.extensions)n[i]===void 0&&(e.userData.gltfExtensions=e.userData.gltfExtensions||{},e.userData.gltfExtensions[i]=t.extensions[i])}function xi(n,e){e.extras!==void 0&&(typeof e.extras=="object"?Object.assign(n.userData,e.extras):console.warn("THREE.GLTFLoader: Ignoring primitive type .extras, "+e.extras))}function kC(n,e,t){let i=!1,r=!1,s=!1;for(let c=0,u=e.length;c<u;c++){const d=e[c];if(d.POSITION!==void 0&&(i=!0),d.NORMAL!==void 0&&(r=!0),d.COLOR_0!==void 0&&(s=!0),i&&r&&s)break}if(!i&&!r&&!s)return Promise.resolve(n);const o=[],a=[],l=[];for(let c=0,u=e.length;c<u;c++){const d=e[c];if(i){const f=d.POSITION!==void 0?t.getDependency("accessor",d.POSITION):n.attributes.position;o.push(f)}if(r){const f=d.NORMAL!==void 0?t.getDependency("accessor",d.NORMAL):n.attributes.normal;a.push(f)}if(s){const f=d.COLOR_0!==void 0?t.getDependency("accessor",d.COLOR_0):n.attributes.color;l.push(f)}}return Promise.all([Promise.all(o),Promise.all(a),Promise.all(l)]).then(function(c){const u=c[0],d=c[1],f=c[2];return i&&(n.morphAttributes.position=u),r&&(n.morphAttributes.normal=d),s&&(n.morphAttributes.color=f),n.morphTargetsRelative=!0,n})}function BC(n,e){if(n.updateMorphTargets(),e.weights!==void 0)for(let t=0,i=e.weights.length;t<i;t++)n.morphTargetInfluences[t]=e.weights[t];if(e.extras&&Array.isArray(e.extras.targetNames)){const t=e.extras.targetNames;if(n.morphTargetInfluences.length===t.length){n.morphTargetDictionary={};for(let i=0,r=t.length;i<r;i++)n.morphTargetDictionary[t[i]]=i}else console.warn("THREE.GLTFLoader: Invalid extras.targetNames length. Ignoring names.")}}function zC(n){let e;const t=n.extensions&&n.extensions[Ve.KHR_DRACO_MESH_COMPRESSION];if(t?e="draco:"+t.bufferView+":"+t.indices+":"+rd(t.attributes):e=n.indices+":"+rd(n.attributes)+":"+n.mode,n.targets!==void 0)for(let i=0,r=n.targets.length;i<r;i++)e+=":"+rd(n.targets[i]);return e}function rd(n){let e="";const t=Object.keys(n).sort();for(let i=0,r=t.length;i<r;i++)e+=t[i]+":"+n[t[i]]+";";return e}function Hf(n){switch(n){case Int8Array:return 1/127;case Uint8Array:return 1/255;case Int16Array:return 1/32767;case Uint16Array:return 1/65535;default:throw new Error("THREE.GLTFLoader: Unsupported normalized accessor component type.")}}function HC(n){return n.search(/\.jpe?g($|\?)/i)>0||n.search(/^data\:image\/jpeg/)===0?"image/jpeg":n.search(/\.webp($|\?)/i)>0||n.search(/^data\:image\/webp/)===0?"image/webp":"image/png"}const VC=new Ue;class GC{constructor(e={},t={}){this.json=e,this.extensions={},this.plugins={},this.options=t,this.cache=new dC,this.associations=new Map,this.primitiveCache={},this.nodeCache={},this.meshCache={refs:{},uses:{}},this.cameraCache={refs:{},uses:{}},this.lightCache={refs:{},uses:{}},this.sourceCache={},this.textureCache={},this.nodeNamesUsed={};let i=!1,r=-1,s=!1,o=-1;if(typeof navigator<"u"){const a=navigator.userAgent;i=/^((?!chrome|android).)*safari/i.test(a)===!0;const l=a.match(/Version\/(\d+)/);r=i&&l?parseInt(l[1],10):-1,s=a.indexOf("Firefox")>-1,o=s?a.match(/Firefox\/([0-9]+)\./)[1]:-1}typeof createImageBitmap>"u"||i&&r<17||s&&o<98?this.textureLoader=new XR(this.options.manager):this.textureLoader=new QR(this.options.manager),this.textureLoader.setCrossOrigin(this.options.crossOrigin),this.textureLoader.setRequestHeader(this.options.requestHeader),this.fileLoader=new ny(this.options.manager),this.fileLoader.setResponseType("arraybuffer"),this.options.crossOrigin==="use-credentials"&&this.fileLoader.setWithCredentials(!0)}setExtensions(e){this.extensions=e}setPlugins(e){this.plugins=e}parse(e,t){const i=this,r=this.json,s=this.extensions;this.cache.removeAll(),this.nodeCache={},this._invokeAll(function(o){return o._markDefs&&o._markDefs()}),Promise.all(this._invokeAll(function(o){return o.beforeRoot&&o.beforeRoot()})).then(function(){return Promise.all([i.getDependencies("scene"),i.getDependencies("animation"),i.getDependencies("camera")])}).then(function(o){const a={scene:o[0][r.scene||0],scenes:o[0],animations:o[1],cameras:o[2],asset:r.asset,parser:i,userData:{}};return Tr(s,a,r),xi(a,r),Promise.all(i._invokeAll(function(l){return l.afterRoot&&l.afterRoot(a)})).then(function(){for(const l of a.scenes)l.updateMatrixWorld();e(a)})}).catch(t)}_markDefs(){const e=this.json.nodes||[],t=this.json.skins||[],i=this.json.meshes||[];for(let r=0,s=t.length;r<s;r++){const o=t[r].joints;for(let a=0,l=o.length;a<l;a++)e[o[a]].isBone=!0}for(let r=0,s=e.length;r<s;r++){const o=e[r];o.mesh!==void 0&&(this._addNodeRef(this.meshCache,o.mesh),o.skin!==void 0&&(i[o.mesh].isSkinnedMesh=!0)),o.camera!==void 0&&this._addNodeRef(this.cameraCache,o.camera)}}_addNodeRef(e,t){t!==void 0&&(e.refs[t]===void 0&&(e.refs[t]=e.uses[t]=0),e.refs[t]++)}_getNodeRef(e,t,i){if(e.refs[t]<=1)return i;const r=i.clone(),s=(o,a)=>{const l=this.associations.get(o);l!=null&&this.associations.set(a,l);for(const[c,u]of o.children.entries())s(u,a.children[c])};return s(i,r),r.name+="_instance_"+e.uses[t]++,r}_invokeOne(e){const t=Object.values(this.plugins);t.push(this);for(let i=0;i<t.length;i++){const r=e(t[i]);if(r)return r}return null}_invokeAll(e){const t=Object.values(this.plugins);t.unshift(this);const i=[];for(let r=0;r<t.length;r++){const s=e(t[r]);s&&i.push(s)}return i}getDependency(e,t){const i=e+":"+t;let r=this.cache.get(i);if(!r){switch(e){case"scene":r=this.loadScene(t);break;case"node":r=this._invokeOne(function(s){return s.loadNode&&s.loadNode(t)});break;case"mesh":r=this._invokeOne(function(s){return s.loadMesh&&s.loadMesh(t)});break;case"accessor":r=this.loadAccessor(t);break;case"bufferView":r=this._invokeOne(function(s){return s.loadBufferView&&s.loadBufferView(t)});break;case"buffer":r=this.loadBuffer(t);break;case"material":r=this._invokeOne(function(s){return s.loadMaterial&&s.loadMaterial(t)});break;case"texture":r=this._invokeOne(function(s){return s.loadTexture&&s.loadTexture(t)});break;case"skin":r=this.loadSkin(t);break;case"animation":r=this._invokeOne(function(s){return s.loadAnimation&&s.loadAnimation(t)});break;case"camera":r=this.loadCamera(t);break;default:if(r=this._invokeOne(function(s){return s!=this&&s.getDependency&&s.getDependency(e,t)}),!r)throw new Error("Unknown type: "+e);break}this.cache.add(i,r)}return r}getDependencies(e){let t=this.cache.get(e);if(!t){const i=this,r=this.json[e+(e==="mesh"?"es":"s")]||[];t=Promise.all(r.map(function(s,o){return i.getDependency(e,o)})),this.cache.add(e,t)}return t}loadBuffer(e){const t=this.json.buffers[e],i=this.fileLoader;if(t.type&&t.type!=="arraybuffer")throw new Error("THREE.GLTFLoader: "+t.type+" buffer type is not supported.");if(t.uri===void 0&&e===0)return Promise.resolve(this.extensions[Ve.KHR_BINARY_GLTF].body);const r=this.options;return new Promise(function(s,o){i.load(qo.resolveURL(t.uri,r.path),s,void 0,function(){o(new Error('THREE.GLTFLoader: Failed to load buffer "'+t.uri+'".'))})})}loadBufferView(e){const t=this.json.bufferViews[e];return this.getDependency("buffer",t.buffer).then(function(i){const r=t.byteLength||0,s=t.byteOffset||0;return i.slice(s,s+r)})}loadAccessor(e){const t=this,i=this.json,r=this.json.accessors[e];if(r.bufferView===void 0&&r.sparse===void 0){const o=nd[r.type],a=ks[r.componentType],l=r.normalized===!0,c=new a(r.count*o);return Promise.resolve(new Rt(c,o,l))}const s=[];return r.bufferView!==void 0?s.push(this.getDependency("bufferView",r.bufferView)):s.push(null),r.sparse!==void 0&&(s.push(this.getDependency("bufferView",r.sparse.indices.bufferView)),s.push(this.getDependency("bufferView",r.sparse.values.bufferView))),Promise.all(s).then(function(o){const a=o[0],l=nd[r.type],c=ks[r.componentType],u=c.BYTES_PER_ELEMENT,d=u*l,f=r.byteOffset||0,p=r.bufferView!==void 0?i.bufferViews[r.bufferView].byteStride:void 0,g=r.normalized===!0;let y,m;if(p&&p!==d){const h=Math.floor(f/p),_="InterleavedBuffer:"+r.bufferView+":"+r.componentType+":"+h+":"+r.count;let v=t.cache.get(_);v||(y=new c(a,h*p,r.count*p/u),v=new Kv(y,p/u),t.cache.add(_,v)),m=new _a(v,l,f%p/u,g)}else a===null?y=new c(r.count*l):y=new c(a,f,r.count*l),m=new Rt(y,l,g);if(r.sparse!==void 0){const h=nd.SCALAR,_=ks[r.sparse.indices.componentType],v=r.sparse.indices.byteOffset||0,x=r.sparse.values.byteOffset||0,R=new _(o[1],v,r.sparse.count*h),A=new c(o[2],x,r.sparse.count*l);a!==null&&(m=new Rt(m.array.slice(),m.itemSize,m.normalized));for(let w=0,b=R.length;w<b;w++){const T=R[w];if(m.setX(T,A[w*l]),l>=2&&m.setY(T,A[w*l+1]),l>=3&&m.setZ(T,A[w*l+2]),l>=4&&m.setW(T,A[w*l+3]),l>=5)throw new Error("THREE.GLTFLoader: Unsupported itemSize in sparse BufferAttribute.")}}return m})}loadTexture(e){const t=this.json,i=this.options,s=t.textures[e].source,o=t.images[s];let a=this.textureLoader;if(o.uri){const l=i.manager.getHandler(o.uri);l!==null&&(a=l)}return this.loadTextureImage(e,s,a)}loadTextureImage(e,t,i){const r=this,s=this.json,o=s.textures[e],a=s.images[t],l=(a.uri||a.bufferView)+":"+o.sampler;if(this.textureCache[l])return this.textureCache[l];const c=this.loadImageSource(t,i).then(function(u){u.flipY=!1,u.name=o.name||a.name||"",u.name===""&&typeof a.uri=="string"&&a.uri.startsWith("data:image/")===!1&&(u.name=a.uri);const f=(s.samplers||{})[o.sampler]||{};return u.magFilter=qg[f.magFilter]||an,u.minFilter=qg[f.minFilter]||Ti,u.wrapS=$g[f.wrapS]||Ks,u.wrapT=$g[f.wrapT]||Ks,r.associations.set(u,{textures:e}),u}).catch(function(){return null});return this.textureCache[l]=c,c}loadImageSource(e,t){const i=this,r=this.json,s=this.options;if(this.sourceCache[e]!==void 0)return this.sourceCache[e].then(d=>d.clone());const o=r.images[e],a=self.URL||self.webkitURL;let l=o.uri||"",c=!1;if(o.bufferView!==void 0)l=i.getDependency("bufferView",o.bufferView).then(function(d){c=!0;const f=new Blob([d],{type:o.mimeType});return l=a.createObjectURL(f),l});else if(o.uri===void 0)throw new Error("THREE.GLTFLoader: Image "+e+" is missing URI and bufferView");const u=Promise.resolve(l).then(function(d){return new Promise(function(f,p){let g=f;t.isImageBitmapLoader===!0&&(g=function(y){const m=new Pt(y);m.needsUpdate=!0,f(m)}),t.load(qo.resolveURL(d,s.path),g,void 0,p)})}).then(function(d){return c===!0&&a.revokeObjectURL(l),xi(d,o),d.userData.mimeType=o.mimeType||HC(o.uri),d}).catch(function(d){throw console.error("THREE.GLTFLoader: Couldn't load texture",l),d});return this.sourceCache[e]=u,u}assignTexture(e,t,i,r){const s=this;return this.getDependency("texture",i.index).then(function(o){if(!o)return null;if(i.texCoord!==void 0&&i.texCoord>0&&(o=o.clone(),o.channel=i.texCoord),s.extensions[Ve.KHR_TEXTURE_TRANSFORM]){const a=i.extensions!==void 0?i.extensions[Ve.KHR_TEXTURE_TRANSFORM]:void 0;if(a){const l=s.associations.get(o);o=s.extensions[Ve.KHR_TEXTURE_TRANSFORM].extendTexture(o,a),s.associations.set(o,l)}}return r!==void 0&&(o.colorSpace=r),e[t]=o,o})}assignFinalMaterial(e){const t=e.geometry;let i=e.material;const r=t.attributes.tangent===void 0,s=t.attributes.color!==void 0,o=t.attributes.normal===void 0;if(e.isPoints){const a="PointsMaterial:"+i.uuid;let l=this.cache.get(a);l||(l=new Qv,qn.prototype.copy.call(l,i),l.color.copy(i.color),l.map=i.map,l.sizeAttenuation=!1,this.cache.add(a,l)),i=l}else if(e.isLine){const a="LineBasicMaterial:"+i.uuid;let l=this.cache.get(a);l||(l=new Qh,qn.prototype.copy.call(l,i),l.color.copy(i.color),l.map=i.map,this.cache.add(a,l)),i=l}if(r||s||o){let a="ClonedMaterial:"+i.uuid+":";r&&(a+="derivative-tangents:"),s&&(a+="vertex-colors:"),o&&(a+="flat-shading:");let l=this.cache.get(a);l||(l=i.clone(),s&&(l.vertexColors=!0),o&&(l.flatShading=!0),r&&(l.normalScale&&(l.normalScale.y*=-1),l.clearcoatNormalScale&&(l.clearcoatNormalScale.y*=-1)),this.cache.add(a,l),this.associations.set(l,this.associations.get(i))),i=l}e.material=i}getMaterialType(){return tp}loadMaterial(e){const t=this,i=this.json,r=this.extensions,s=i.materials[e];let o;const a={},l=s.extensions||{},c=[];if(l[Ve.KHR_MATERIALS_UNLIT]){const d=r[Ve.KHR_MATERIALS_UNLIT];o=d.getMaterialType(),c.push(d.extendParams(a,s,t))}else{const d=s.pbrMetallicRoughness||{};if(a.color=new Pe(1,1,1),a.opacity=1,Array.isArray(d.baseColorFactor)){const f=d.baseColorFactor;a.color.setRGB(f[0],f[1],f[2],Vt),a.opacity=f[3]}d.baseColorTexture!==void 0&&c.push(t.assignTexture(a,"map",d.baseColorTexture,Qt)),a.metalness=d.metallicFactor!==void 0?d.metallicFactor:1,a.roughness=d.roughnessFactor!==void 0?d.roughnessFactor:1,d.metallicRoughnessTexture!==void 0&&(c.push(t.assignTexture(a,"metalnessMap",d.metallicRoughnessTexture)),c.push(t.assignTexture(a,"roughnessMap",d.metallicRoughnessTexture))),o=this._invokeOne(function(f){return f.getMaterialType&&f.getMaterialType(e)}),c.push(Promise.all(this._invokeAll(function(f){return f.extendMaterialParams&&f.extendMaterialParams(e,a)})))}s.doubleSided===!0&&(a.side=ii);const u=s.alphaMode||id.OPAQUE;if(u===id.BLEND?(a.transparent=!0,a.depthWrite=!1):(a.transparent=!1,u===id.MASK&&(a.alphaTest=s.alphaCutoff!==void 0?s.alphaCutoff:.5)),s.normalTexture!==void 0&&o!==Ai&&(c.push(t.assignTexture(a,"normalMap",s.normalTexture)),a.normalScale=new we(1,1),s.normalTexture.scale!==void 0)){const d=s.normalTexture.scale;a.normalScale.set(d,d)}if(s.occlusionTexture!==void 0&&o!==Ai&&(c.push(t.assignTexture(a,"aoMap",s.occlusionTexture)),s.occlusionTexture.strength!==void 0&&(a.aoMapIntensity=s.occlusionTexture.strength)),s.emissiveFactor!==void 0&&o!==Ai){const d=s.emissiveFactor;a.emissive=new Pe().setRGB(d[0],d[1],d[2],Vt)}return s.emissiveTexture!==void 0&&o!==Ai&&c.push(t.assignTexture(a,"emissiveMap",s.emissiveTexture,Qt)),Promise.all(c).then(function(){const d=new o(a);return s.name&&(d.name=s.name),xi(d,s),t.associations.set(d,{materials:e}),s.extensions&&Tr(r,d,s),d})}createUniqueName(e){const t=nt.sanitizeNodeName(e||"");return t in this.nodeNamesUsed?t+"_"+ ++this.nodeNamesUsed[t]:(this.nodeNamesUsed[t]=0,t)}loadGeometries(e){const t=this,i=this.extensions,r=this.primitiveCache;function s(a){return i[Ve.KHR_DRACO_MESH_COMPRESSION].decodePrimitive(a,t).then(function(l){return Zg(l,a,t)})}const o=[];for(let a=0,l=e.length;a<l;a++){const c=e[a],u=zC(c),d=r[u];if(d)o.push(d.promise);else{let f;c.extensions&&c.extensions[Ve.KHR_DRACO_MESH_COMPRESSION]?f=s(c):f=Zg(new hn,c,t),r[u]={primitive:c,promise:f},o.push(f)}}return Promise.all(o)}loadMesh(e){const t=this,i=this.json,r=this.extensions,s=i.meshes[e],o=s.primitives,a=[];for(let l=0,c=o.length;l<c;l++){const u=o[l].material===void 0?OC(this.cache):this.getDependency("material",o[l].material);a.push(u)}return a.push(t.loadGeometries(o)),Promise.all(a).then(function(l){const c=l.slice(0,l.length-1),u=l[l.length-1],d=[];for(let p=0,g=u.length;p<g;p++){const y=u[p],m=o[p];let h;const _=c[p];if(m.mode===An.TRIANGLES||m.mode===An.TRIANGLE_STRIP||m.mode===An.TRIANGLE_FAN||m.mode===void 0)h=s.isSkinnedMesh===!0?new TR(y,_):new tn(y,_),h.isSkinnedMesh===!0&&h.normalizeSkinWeights(),m.mode===An.TRIANGLE_STRIP?h.geometry=Yg(h.geometry,Pv):m.mode===An.TRIANGLE_FAN&&(h.geometry=Yg(h.geometry,Nf));else if(m.mode===An.LINES)h=new CR(y,_);else if(m.mode===An.LINE_STRIP)h=new Wc(y,_);else if(m.mode===An.LINE_LOOP)h=new bR(y,_);else if(m.mode===An.POINTS)h=new Jv(y,_);else throw new Error("THREE.GLTFLoader: Primitive mode unsupported: "+m.mode);Object.keys(h.geometry.morphAttributes).length>0&&BC(h,s),h.name=t.createUniqueName(s.name||"mesh_"+e),xi(h,s),m.extensions&&Tr(r,h,m),t.assignFinalMaterial(h),d.push(h)}for(let p=0,g=d.length;p<g;p++)t.associations.set(d[p],{meshes:e,primitives:p});if(d.length===1)return s.extensions&&Tr(r,d[0],s),d[0];const f=new ri;s.extensions&&Tr(r,f,s),t.associations.set(f,{meshes:e});for(let p=0,g=d.length;p<g;p++)f.add(d[p]);return f})}loadCamera(e){let t;const i=this.json.cameras[e],r=i[i.type];if(!r){console.warn("THREE.GLTFLoader: Missing camera parameters.");return}return i.type==="perspective"?t=new Jt(uE.radToDeg(r.yfov),r.aspectRatio||1,r.znear||1,r.zfar||2e6):i.type==="orthographic"&&(t=new qh(-r.xmag,r.xmag,r.ymag,-r.ymag,r.znear,r.zfar)),i.name&&(t.name=this.createUniqueName(i.name)),xi(t,i),Promise.resolve(t)}loadSkin(e){const t=this.json.skins[e],i=[];for(let r=0,s=t.joints.length;r<s;r++)i.push(this._loadNodeShallow(t.joints[r]));return t.inverseBindMatrices!==void 0?i.push(this.getDependency("accessor",t.inverseBindMatrices)):i.push(null),Promise.all(i).then(function(r){const s=r.pop(),o=r,a=[],l=[];for(let c=0,u=o.length;c<u;c++){const d=o[c];if(d){a.push(d);const f=new Ue;s!==null&&f.fromArray(s.array,c*16),l.push(f)}else console.warn('THREE.GLTFLoader: Joint "%s" could not be found.',t.joints[c])}return new Zh(a,l)})}loadAnimation(e){const t=this.json,i=this,r=t.animations[e],s=r.name?r.name:"animation_"+e,o=[],a=[],l=[],c=[],u=[];for(let d=0,f=r.channels.length;d<f;d++){const p=r.channels[d],g=r.samplers[p.sampler],y=p.target,m=y.node,h=r.parameters!==void 0?r.parameters[g.input]:g.input,_=r.parameters!==void 0?r.parameters[g.output]:g.output;y.node!==void 0&&(o.push(this.getDependency("node",m)),a.push(this.getDependency("accessor",h)),l.push(this.getDependency("accessor",_)),c.push(g),u.push(y))}return Promise.all([Promise.all(o),Promise.all(a),Promise.all(l),Promise.all(c),Promise.all(u)]).then(function(d){const f=d[0],p=d[1],g=d[2],y=d[3],m=d[4],h=[];for(let _=0,v=f.length;_<v;_++){const x=f[_],R=p[_],A=g[_],w=y[_],b=m[_];if(x===void 0)continue;x.updateMatrix&&x.updateMatrix();const T=i._createAnimationTracks(x,R,A,w,b);if(T)for(let M=0;M<T.length;M++)h.push(T[M])}return new kR(s,void 0,h)})}createNodeMesh(e){const t=this.json,i=this,r=t.nodes[e];return r.mesh===void 0?null:i.getDependency("mesh",r.mesh).then(function(s){const o=i._getNodeRef(i.meshCache,r.mesh,s);return r.weights!==void 0&&o.traverse(function(a){if(a.isMesh)for(let l=0,c=r.weights.length;l<c;l++)a.morphTargetInfluences[l]=r.weights[l]}),o})}loadNode(e){const t=this.json,i=this,r=t.nodes[e],s=i._loadNodeShallow(e),o=[],a=r.children||[];for(let c=0,u=a.length;c<u;c++)o.push(i.getDependency("node",a[c]));const l=r.skin===void 0?Promise.resolve(null):i.getDependency("skin",r.skin);return Promise.all([s,Promise.all(o),l]).then(function(c){const u=c[0],d=c[1],f=c[2];f!==null&&u.traverse(function(p){p.isSkinnedMesh&&p.bind(f,VC)});for(let p=0,g=d.length;p<g;p++)u.add(d[p]);return u})}_loadNodeShallow(e){const t=this.json,i=this.extensions,r=this;if(this.nodeCache[e]!==void 0)return this.nodeCache[e];const s=t.nodes[e],o=s.name?r.createUniqueName(s.name):"",a=[],l=r._invokeOne(function(c){return c.createNodeMesh&&c.createNodeMesh(e)});return l&&a.push(l),s.camera!==void 0&&a.push(r.getDependency("camera",s.camera).then(function(c){return r._getNodeRef(r.cameraCache,s.camera,c)})),r._invokeAll(function(c){return c.createNodeAttachment&&c.createNodeAttachment(e)}).forEach(function(c){a.push(c)}),this.nodeCache[e]=Promise.all(a).then(function(c){let u;if(s.isBone===!0?u=new $v:c.length>1?u=new ri:c.length===1?u=c[0]:u=new dt,u!==c[0])for(let d=0,f=c.length;d<f;d++)u.add(c[d]);if(s.name&&(u.userData.name=s.name,u.name=o),xi(u,s),s.extensions&&Tr(i,u,s),s.matrix!==void 0){const d=new Ue;d.fromArray(s.matrix),u.applyMatrix4(d)}else s.translation!==void 0&&u.position.fromArray(s.translation),s.rotation!==void 0&&u.quaternion.fromArray(s.rotation),s.scale!==void 0&&u.scale.fromArray(s.scale);return r.associations.has(u)||r.associations.set(u,{}),r.associations.get(u).nodes=e,u}),this.nodeCache[e]}loadScene(e){const t=this.extensions,i=this.json.scenes[e],r=this,s=new ri;i.name&&(s.name=r.createUniqueName(i.name)),xi(s,i),i.extensions&&Tr(t,s,i);const o=i.nodes||[],a=[];for(let l=0,c=o.length;l<c;l++)a.push(r.getDependency("node",o[l]));return Promise.all(a).then(function(l){for(let u=0,d=l.length;u<d;u++)s.add(l[u]);const c=u=>{const d=new Map;for(const[f,p]of r.associations)(f instanceof qn||f instanceof Pt)&&d.set(f,p);return u.traverse(f=>{const p=r.associations.get(f);p!=null&&d.set(f,p)}),d};return r.associations=c(s),s})}_createAnimationTracks(e,t,i,r,s){const o=[],a=e.name?e.name:e.uuid,l=[];Xi[s.path]===Xi.weights?e.traverse(function(f){f.morphTargetInfluences&&l.push(f.name?f.name:f.uuid)}):l.push(a);let c;switch(Xi[s.path]){case Xi.weights:c=Js;break;case Xi.rotation:c=eo;break;case Xi.position:case Xi.scale:c=to;break;default:switch(i.itemSize){case 1:c=Js;break;case 2:case 3:default:c=to;break}break}const u=r.interpolation!==void 0?FC[r.interpolation]:ma,d=this._getArrayFromAccessor(i);for(let f=0,p=l.length;f<p;f++){const g=new c(l[f]+"."+Xi[s.path],t.array,d,u);r.interpolation==="CUBICSPLINE"&&this._createCubicSplineTrackInterpolant(g),o.push(g)}return o}_getArrayFromAccessor(e){let t=e.array;if(e.normalized){const i=Hf(t.constructor),r=new Float32Array(t.length);for(let s=0,o=t.length;s<o;s++)r[s]=t[s]*i;t=r}return t}_createCubicSplineTrackInterpolant(e){e.createInterpolant=function(i){const r=this instanceof eo?UC:ry;return new r(this.times,this.values,this.getValueSize()/3,i)},e.createInterpolant.isInterpolantFactoryMethodGLTFCubicSpline=!0}}function WC(n,e,t){const i=e.attributes,r=new Fi;if(i.POSITION!==void 0){const a=t.json.accessors[i.POSITION],l=a.min,c=a.max;if(l!==void 0&&c!==void 0){if(r.set(new L(l[0],l[1],l[2]),new L(c[0],c[1],c[2])),a.normalized){const u=Hf(ks[a.componentType]);r.min.multiplyScalar(u),r.max.multiplyScalar(u)}}else{console.warn("THREE.GLTFLoader: Missing min/max properties for accessor POSITION.");return}}else return;const s=e.targets;if(s!==void 0){const a=new L,l=new L;for(let c=0,u=s.length;c<u;c++){const d=s[c];if(d.POSITION!==void 0){const f=t.json.accessors[d.POSITION],p=f.min,g=f.max;if(p!==void 0&&g!==void 0){if(l.setX(Math.max(Math.abs(p[0]),Math.abs(g[0]))),l.setY(Math.max(Math.abs(p[1]),Math.abs(g[1]))),l.setZ(Math.max(Math.abs(p[2]),Math.abs(g[2]))),f.normalized){const y=Hf(ks[f.componentType]);l.multiplyScalar(y)}a.max(l)}else console.warn("THREE.GLTFLoader: Missing min/max properties for accessor POSITION.")}}r.expandByVector(a)}n.boundingBox=r;const o=new li;r.getCenter(o.center),o.radius=r.min.distanceTo(r.max)/2,n.boundingSphere=o}function Zg(n,e,t){const i=e.attributes,r=[];function s(o,a){return t.getDependency("accessor",o).then(function(l){n.setAttribute(a,l)})}for(const o in i){const a=zf[o]||o.toLowerCase();a in n.attributes||r.push(s(i[o],a))}if(e.indices!==void 0&&!n.index){const o=t.getDependency("accessor",e.indices).then(function(a){n.setIndex(a)});r.push(o)}return qe.workingColorSpace!==Vt&&"COLOR_0"in i&&console.warn(`THREE.GLTFLoader: Converting vertex colors from "srgb-linear" to "${qe.workingColorSpace}" not supported.`),xi(n,e),WC(n,e,t),Promise.all(r).then(function(){return e.targets!==void 0?kC(n,e.targets,t):n})}const XC="/ui/assets/brain-BKX8gfr4.glb",jC=`
attribute vec3 aOffset;
attribute float aRotation;
attribute float aSize;
attribute vec3 aColor;
attribute float aSeed;
uniform vec3 uPointer;
uniform float uHover;
uniform float uTime;
uniform float uIntro;
varying vec3 vColor;
varying float vAlpha;

void main() {
  float d = distance(uPointer, aOffset);
  float c = smoothstep(0.45, 0.08, d);
  float scale = (aSize + c * 10.0 * uHover) * uIntro;
  float drift = uTime * (0.2 + aSeed * 0.3);
  vec3 pos = aOffset;
  pos.x += sin(drift + aSeed * 6.28) * 0.004 * aRotation;
  pos.y += cos(drift * 0.7 + aSeed * 3.14) * 0.004 * aRotation;
  pos.z += sin(drift * 0.5 + aSeed * 4.71) * 0.004 * aRotation;
  vec4 mvPosition = modelViewMatrix * vec4(pos, 1.0);
  gl_Position = projectionMatrix * mvPosition;
  gl_PointSize = scale * (500.0 / -mvPosition.z);
  float flicker = 0.72 + 0.28 * sin(uTime * 2.6 + aSeed * 39.0);
  vColor = mix(aColor, vec3(1.0), c * uHover * 0.9);
  vAlpha = flicker * (0.65 + 0.35 * c * uHover);
}
`,YC=`
varying vec3 vColor;
varying float vAlpha;
void main() {
  float dist = distance(gl_PointCoord, vec2(0.5));
  if (dist > 0.5) discard;
  float soft = 1.0 - smoothstep(0.35, 0.5, dist);
  gl_FragColor = vec4(vColor, vAlpha * soft);
}
`,Qg={nodeBase:11032055,nodeHot:16777215,nodeUnfocus:5974922,edgeRest:7153881,edgeActive:12616956,edgeDim:2756688,particles:[8141549,11032055,7153881,12616956,16777215]},Jg={nodeBase:8141549,nodeHot:4988309,nodeUnfocus:12891645,edgeRest:8141549,edgeActive:7153881,edgeDim:15324671,particles:[8141549,7153881,9647082,4988309,11032055]},e_=2500,t_=3e3,KC=300,sd=.85,qC=.35,$C=.05,ZC=.02,n_=1.5,QC=.01,od=.01,JC=5e-4;class eb{constructor(e,t){ue(this,"container");ue(this,"opts");ue(this,"palette");ue(this,"renderer");ue(this,"scene",new MR);ue(this,"camera");ue(this,"brainGroup",new ri);ue(this,"nodeGroup",new ri);ue(this,"edgeGroup",new ri);ue(this,"proxy");ue(this,"uniforms",{uPointer:{value:new L(999,999,999)},uHover:{value:0},uTime:{value:0},uIntro:{value:0}});ue(this,"pointerTarget",new L(999,999,999));ue(this,"hoverTarget",0);ue(this,"focusPulse",null);ue(this,"verts",new Float32Array(0));ue(this,"norms",new Float32Array(0));ue(this,"particleColorIndex",[]);ue(this,"particleColorAttr");ue(this,"sphereGeo",new ep(1,10,10));ue(this,"haloTexture");ue(this,"nodeObjects",[]);ue(this,"nodeMap",new Map);ue(this,"haloMap",new Map);ue(this,"labelMap",new Map);ue(this,"links",[]);ue(this,"atomsById",new Map);ue(this,"regionVerts",{left:[],right:[],stem:[]});ue(this,"usedVerts",new Set);ue(this,"degrees",new Map);ue(this,"layoutMode","regions");ue(this,"_solarSunId","");ue(this,"_solarPlanets",new Map);ue(this,"_solarMoons",new Map);ue(this,"_solarAsteroids",new Map);ue(this,"forcePositions",new Map);ue(this,"_forcePosArr",new Float32Array(0));ue(this,"_forceVelArr",new Float32Array(0));ue(this,"_forceIds",[]);ue(this,"_forceIndex",new Map);ue(this,"_forceLinkPairs",[]);ue(this,"_forceSimHandle",null);ue(this,"_forceIterations",0);ue(this,"connectionThreshold",0);ue(this,"showSynapses",!0);ue(this,"showLabels",!1);ue(this,"showRegions",!1);ue(this,"zoomNode",null);ue(this,"manualDistance",null);ue(this,"raycaster",new cC);ue(this,"pointerNdc",new we);ue(this,"parallax",new we);ue(this,"parallaxTarget",new we);ue(this,"yaw",0);ue(this,"pitch",0);ue(this,"distance",1.75);ue(this,"dragging",!1);ue(this,"last",null);ue(this,"hover",null);ue(this,"clock",new JR);ue(this,"raf",0);ue(this,"resizeObserver");ue(this,"disposed",!1);ue(this,"onPointerMove",e=>{var i;if(this.updateNdc(e),this.dragging&&this.last){this.yaw+=(e.clientX-this.last.x)*.008,this.pitch=Math.max(-.8,Math.min(.8,this.pitch+(e.clientY-this.last.y)*.008)),this.last={x:e.clientX,y:e.clientY};return}this.raycaster.setFromCamera(this.pointerNdc,this.camera);const t=(i=this.raycaster.intersectObjects(this.nodeObjects,!1)[0])==null?void 0:i.object;if(t){this.showHover(t,e);return}if(this.hover&&this.clearHover(),this.proxy){const r=this.raycaster.intersectObject(this.proxy,!1)[0];if(r){this.pointerTarget.copy(this.brainGroup.worldToLocal(r.point.clone())),this.hoverTarget=Math.max(this.hoverTarget,.65);return}}this.hoverTarget=0});ue(this,"onPointerDown",e=>{this.dragging=!0,this.last={x:e.clientX,y:e.clientY}});ue(this,"onPointerUp",()=>{this.dragging=!1,this.last=null});ue(this,"onWheel",e=>{e.preventDefault(),this.manualDistance=null,this.distance=Math.max(1,Math.min(5,this.distance+e.deltaY*.0012))});ue(this,"onClick",e=>{var r;this.updateNdc(e),this.raycaster.setFromCamera(this.pointerNdc,this.camera);const t=(r=this.raycaster.intersectObjects(this.nodeObjects,!1)[0])==null?void 0:r.object,i=t&&this.atomsById.get(t.userData.atomId);i?(this.zoomNode===t?this.zoomOut():this.zoomNode=t,this.opts.onSelectAtom(i)):this.zoomOut()});ue(this,"onKeydown",e=>{e.key==="Escape"&&(this.clearHover(),this.hoverTarget=0,this.zoomOut())});ue(this,"loop",()=>{if(this.disposed)return;const e=Math.min(this.clock.getDelta(),.1),t=performance.now();if(this.focusPulse&&(t<this.focusPulse.until?(this.pointerTarget.copy(this.focusPulse.pos),this.hoverTarget=1):(this.focusPulse=null,this.hoverTarget=0)),this.opts.reducedMotion||(this.uniforms.uTime.value+=e,!this.dragging&&!this.zoomNode&&(this.brainGroup.rotation.y+=e*.05),this.uniforms.uIntro.value<1&&(this.uniforms.uIntro.value=Math.min(1,this.uniforms.uIntro.value+e*.8))),this.uniforms.uPointer.value.lerp(this.pointerTarget,.18),this.uniforms.uHover.value+=(this.hoverTarget-this.uniforms.uHover.value)*.2,this.parallax.lerp(this.parallaxTarget,.06),this.manualDistance!==null&&(this.distance+=(this.manualDistance-this.distance)*.15),this.zoomNode&&this.zoomNode.visible){const i=new L;this.zoomNode.getWorldPosition(i);const r=new L().subVectors(this.camera.position,i);r.lengthSq()<1e-4&&r.set(0,0,1),r.normalize();const s=i.clone().add(r.multiplyScalar(.35));this.camera.position.lerp(s,.06),this.camera.lookAt(i)}else this.camera.position.set(Math.sin(this.yaw)*this.distance+this.parallax.x,Math.sin(this.pitch)*this.distance*.6+this.parallax.y,Math.cos(this.yaw)*this.distance),this.camera.lookAt(0,0,0);this.renderer.render(this.scene,this.camera),this.raf=requestAnimationFrame(this.loop)});this.container=e,this.opts=t,this.palette=t.theme==="light"?Jg:Qg,this.haloTexture=this.makeHaloTexture();try{this.renderer=new SR({antialias:!0,alpha:!0})}catch{t.onError("WebGL is unavailable in this browser.");return}this.renderer.setPixelRatio(Math.min(window.devicePixelRatio,2)),this.renderer.setClearColor(0,0),e.appendChild(this.renderer.domElement),this.camera=new Jt(45,1,.05,60),this.brainGroup.add(this.nodeGroup,this.edgeGroup),this.brainGroup.position.y=.08,this.scene.add(this.brainGroup),this.uniforms.uIntro.value=t.reducedMotion?1:0,this.loadModel(),this.addListeners(),this.resizeObserver=new ResizeObserver(()=>this.resize()),this.resizeObserver.observe(e),this.resize(),this.loop()}loadModel(){new uC().load(XC,e=>{if(this.disposed)return;let t=null;if(e.scene.traverse(a=>{!t&&a.isMesh&&(t=a)}),!t){this.opts.onError("Brain model failed to parse.");return}const r=t.geometry,s=r.getAttribute("position"),o=r.getAttribute("normal");this.verts=new Float32Array(s.array),this.norms=o?new Float32Array(o.array):this.deriveNormals(this.verts),this.classifyVertices(),this.proxy=new tn(r,new Ai),this.proxy.visible=!1,this.brainGroup.add(this.proxy),this.buildParticles(s.count),this.opts.onReady()},void 0,()=>this.opts.onError("Brain model failed to load."))}deriveNormals(e){const t=new Float32Array(e.length),i=new L;for(let r=0;r<e.length;r+=3)i.set(e[r],e[r+1],e[r+2]).normalize(),t[r]=i.x,t[r+1]=i.y,t[r+2]=i.z;return t}classifyVertices(){if(this.regionVerts={left:[],right:[],stem:[]},!(this.verts.length/3))return;let t=1/0,i=-1/0,r=1/0,s=-1/0;for(let d=0;d<this.verts.length;d+=3)t=Math.min(t,this.verts[d]),i=Math.max(i,this.verts[d]),r=Math.min(r,this.verts[d+1]),s=Math.max(s,this.verts[d+1]);const o=i-t,a=s-r,l=(t+i)/2,c=r+.13*a,u=.12*o;for(let d=0,f=0;d<this.verts.length;d+=3,f++){const p=this.verts[d];this.verts[d+1]<c&&Math.abs(p-l)<u?this.regionVerts.stem.push(f):p<l?this.regionVerts.left.push(f):this.regionVerts.right.push(f)}}vertexForRegion(e,t){const i=this.verts.length/3;if(!i)return new L;const r=[t,t==="left"?"right":"left","stem"],s=Mm(e.id);for(const o of r){const a=this.regionVerts[o];if(!a.length)continue;const l=s%a.length;for(let c=0;c<a.length;c++){const u=a[(l+c)%a.length];if(!this.usedVerts.has(u))return this.usedVerts.add(u),this.vertexFromIndex(u,s)}}return this.vertexFromIndex(s%i*3,s)}vertexFromIndex(e,t){const i=e*3,r=new L(this.verts[i],this.verts[i+1],this.verts[i+2]),s=new L(this.norms[i],this.norms[i+1],this.norms[i+2]),o=((t>>8)%100/100-.5)*.03,a=((t>>16)%100/100-.5)*.03;return r.add(s.multiplyScalar(.018+o)).addScalar(a*.01)}buildParticles(e){const t=new hn,i=new Float32Array(e),r=new Float32Array(e),s=new Float32Array(e*3),o=new Float32Array(e),a=new Pe;this.particleColorIndex=[];for(let u=0;u<e;u++){i[u]=Math.random()*2-1,r[u]=1+Math.random()*4,o[u]=Math.random();const d=Math.random()<.08?this.palette.particles.length-1:Math.floor(Math.random()*(this.palette.particles.length-1));this.particleColorIndex.push(d),a.setHex(this.palette.particles[d]),s[u*3]=a.r,s[u*3+1]=a.g,s[u*3+2]=a.b}t.setAttribute("aOffset",new Rt(this.verts,3)),t.setAttribute("aRotation",new Rt(i,1)),t.setAttribute("aSize",new Rt(r,1)),this.particleColorAttr=new Rt(s,3),t.setAttribute("aColor",this.particleColorAttr),t.setAttribute("aSeed",new Rt(o,1));const l=new Di({vertexShader:jC,fragmentShader:YC,uniforms:this.uniforms,transparent:!0,depthWrite:!1,blending:mc}),c=new Jv(t,l);c.frustumCulled=!1,this.brainGroup.add(c)}regionColor(e){return e==="left"?6333946:e==="right"?11032055:16777215}setData(e,t){if(this.atomsById=new Map(e.map(c=>[c.id,c])),this.links=t,this.hover=null,this.zoomNode=null,this.degrees=new Map,t.forEach(c=>{this.degrees.set(c.a,(this.degrees.get(c.a)||0)+1),this.degrees.set(c.b,(this.degrees.get(c.b)||0)+1)}),this.nodeGroup.clear(),this.edgeGroup.clear(),this.nodeObjects=[],this.nodeMap=new Map,this.haloMap=new Map,this.labelMap=new Map,this.usedVerts=new Set,!this.verts.length)return;const i=e.slice(0,e_),r=[...i].sort((c,u)=>(this.degrees.get(u.id)||0)-(this.degrees.get(c.id)||0)),s=r.length,o=Math.max(1,Math.ceil(s*.1)),a=Math.ceil(s*.4),l=new Map;r.slice(0,o).forEach(c=>l.set(c.id,"stem")),r.slice(o,o+a).forEach(c=>l.set(c.id,"right")),r.slice(o+a).forEach(c=>l.set(c.id,"left")),i.forEach(c=>{const u=l.get(c.id)||"left",d=this.degrees.get(c.id)||0,f=.006+Math.min(d,20)*.0012+Math.min(c.score,1)*.003,p=new tn(this.sphereGeo,this.nodeMaterial(!1));p.scale.setScalar(f),p.position.copy(this.vertexForRegion(c,u)),p.userData.atomId=c.id,p.userData.region=u;const g=new Tg(this.haloMaterial(!1));g.scale.setScalar(f*3.4),g.position.copy(p.position),g.raycast=()=>{};const y=new Tg(new Ff({map:this.makeLabelTexture(c.title),transparent:!0,depthWrite:!1,depthTest:!0}));y.scale.setScalar(.09),y.position.copy(p.position),y.position.y+=f*2.2,y.visible=!1,y.raycast=()=>{},this.nodeGroup.add(p,g,y),this.nodeObjects.push(p),this.nodeMap.set(c.id,p),this.haloMap.set(c.id,g),this.labelMap.set(c.id,y)}),t.slice().sort((c,u)=>u.strength-c.strength).slice(0,t_).forEach(c=>{const u=this.nodeMap.get(c.a),d=this.nodeMap.get(c.b);if(!u||!d)return;const f=[u.position.clone()],p=4;for(let h=1;h<=p;h++){const _=h/(p+1),v=u.position.clone().lerp(d.position,_);if(this.proxy){const x=new L;this.brainGroup.localToWorld(x);const R=v.clone();this.brainGroup.localToWorld(R);const A=R.clone().sub(x).normalize();this.raycaster.set(x,A);const w=this.raycaster.intersectObject(this.proxy,!1)[0];if(w){const b=w.point.clone();this.brainGroup.worldToLocal(b);const T=b.clone().normalize();b.add(T.multiplyScalar(.005)),f.push(b)}else{const b=v.clone().normalize().multiplyScalar(v.length()+.01);f.push(b)}}else{const x=v.length()*.18;f.push(v.add(v.clone().normalize().multiplyScalar(x)))}}f.push(d.position.clone());const g=new LR(f),y=new hn().setFromPoints(g.getPoints(28)),m=new Wc(y,new Qh({color:this.palette.edgeRest,transparent:!0,opacity:.2,depthWrite:!1}));m.userData=c,this.edgeGroup.add(m)}),this.applyFilters(),this.applyRegionColors(),this.computeForceLayout()}makeLabelTexture(e){const t=document.createElement("canvas"),i=t.getContext("2d"),r=34,s=`${r}px "JetBrains Mono", monospace`;i.font=s;const o=10,a=i.measureText(e);t.width=Math.ceil(a.width+o*2),t.height=r+o,i.font=s,i.textBaseline="top",i.fillStyle="rgba(244, 240, 255, 0.92)",i.fillText(e,o,o*.5);const l=new Bg(t);return l.minFilter=an,l}setConnectionThreshold(e){this.connectionThreshold=e,this.applyFilters()}setZoomLevel(e){this.manualDistance=1+e/100*4}setShowSynapses(e){this.showSynapses=e,this.applyFilters()}setShowLabels(e){this.showLabels=e,this.applyFilters()}setShowRegions(e){this.showRegions=e,this.applyRegionColors()}zoomToAtom(e){const t=this.nodeMap.get(e.id);t&&(this.zoomNode=t)}zoomOut(){this.zoomNode&&(this.zoomNode=null,this.opts.onZoomOut(),this.clearHover())}randomConnection(){if(!this.nodeObjects.length)return;const e=this.nodeObjects.filter(o=>o.visible),t=e.length?e:this.nodeObjects,i=t[Math.floor(Math.random()*t.length)],r=i.userData.atomId,s=this.atomsById.get(r);s&&(this.highlightConnections(i),this.zoomNode=i,this.opts.onSelectAtom(s))}applyFilters(){this.nodeObjects.forEach(e=>{const t=e.userData.atomId,r=(this.degrees.get(t)||0)>=this.connectionThreshold;e.visible=r;const s=this.haloMap.get(t);s&&(s.visible=r);const o=this.labelMap.get(t);o&&(o.visible=r&&this.showLabels)}),this.edgeGroup.children.forEach(e=>{const t=e.userData,i=this.degrees.get(t.a)||0,r=this.degrees.get(t.b)||0;e.visible=this.showSynapses&&i>=this.connectionThreshold&&r>=this.connectionThreshold})}applyRegionColors(){this.showRegions?this.nodeObjects.forEach(e=>{const t=e.userData.region,i=this.regionColor(t);e.material.color.setHex(i),this.haloMap.get(e.userData.atomId).material.color.setHex(i)}):this.nodeObjects.forEach(e=>{e.material.color.setHex(this.palette.nodeBase),this.haloMap.get(e.userData.atomId).material.color.setHex(this.palette.nodeBase)})}nodeMaterial(e){return new Ai({color:e?this.palette.nodeHot:this.palette.nodeBase,transparent:!0,opacity:e?1:.85})}haloMaterial(e){return new Ff({map:this.haloTexture,color:e?this.palette.nodeHot:this.palette.nodeBase,transparent:!0,opacity:e?.7:.32,depthWrite:!1,blending:mc})}makeHaloTexture(){const t=document.createElement("canvas");t.width=t.height=64;const i=t.getContext("2d"),r=i.createRadialGradient(64/2,64/2,0,64/2,64/2,64/2);return r.addColorStop(0,"rgba(255,255,255,1)"),r.addColorStop(.35,"rgba(255,255,255,.35)"),r.addColorStop(1,"rgba(255,255,255,0)"),i.fillStyle=r,i.fillRect(0,0,64,64),new Bg(t)}highlightConnections(e){this.hover=e;const t=e.userData.atomId,i=new Set;this.links.forEach(r=>{r.a===t?i.add(r.b):r.b===t&&i.add(r.a)}),this.nodeObjects.forEach(r=>{const s=r.userData.atomId,o=r.material,a=this.haloMap.get(s).material;if(r===e)o.color.setHex(this.palette.nodeHot),o.opacity=1,a.color.setHex(this.palette.nodeHot),a.opacity=.7;else if(i.has(s)){const l=this.showRegions?this.regionColor(r.userData.region):this.palette.nodeBase;o.color.setHex(l),o.opacity=.9,a.color.setHex(l),a.opacity=.32}else o.color.setHex(this.palette.nodeUnfocus),o.opacity=.3,a.opacity=.1}),this.edgeGroup.children.forEach(r=>{const s=r.userData,o=s.a===t||s.b===t,a=r.material;a.opacity=o?.9:.04,a.color.setHex(o?this.palette.edgeActive:this.palette.edgeDim)}),this.pointerTarget.copy(e.position),this.hoverTarget=1}showHover(e,t){this.highlightConnections(e);const i=this.atomsById.get(e.userData.atomId);i&&this.opts.onHoverAtom(i,t.clientX,t.clientY)}clearHover(){this.hover&&(this.hover=null,this.nodeObjects.forEach(e=>{const t=e.material,i=this.haloMap.get(e.userData.atomId).material,r=this.showRegions?this.regionColor(e.userData.region):this.palette.nodeBase;t.color.setHex(r),t.opacity=.85,i.color.setHex(r),i.opacity=.32}),this.edgeGroup.children.forEach(e=>{const t=e.material;t.opacity=.2,t.color.setHex(this.palette.edgeRest)}),this.opts.onClearHover())}focusAtom(e){const t=this.nodeMap.get(e.id);t&&(this.focusPulse={pos:t.position.clone(),until:performance.now()+1400})}setLayout(e){var i;if(this.layoutMode===e)return;this.layoutMode=e;const t=[...this.atomsById.values()];if(e==="solar"){const r=[...t].sort((d,f)=>(this.degrees.get(f.id)||0)-(this.degrees.get(d.id)||0));this._solarSunId=((i=r[0])==null?void 0:i.id)??"";const s=14,o=r.slice(1,s+1);this._solarPlanets=new Map,o.forEach((d,f)=>{const p=.8+f/Math.max(o.length-1,1)*3.5,g=f/o.length*Math.PI*2,y=(Math.random()-.5)*.6;this._solarPlanets.set(d.id,{orbitR:p,angle:g,tilt:y})});const a=new Set(o.map(d=>d.id)),l=new Set([this._solarSunId,...a]);this._solarMoons=new Map,this._solarAsteroids=new Map;const c=new Map;r.slice(s+1).forEach(d=>{const f=this.links.find(p=>p.a===d.id&&a.has(p.b)||p.b===d.id&&a.has(p.a));if(f){const p=a.has(f.b)?f.b:f.a,g=(c.get(p)||0)+1;c.set(p,g),this._solarMoons.set(d.id,{planetId:p,moonR:.15+g%5*.08,moonAngle:g*2.4%(Math.PI*2)}),l.add(d.id)}});let u=0;t.forEach(d=>{if(l.has(d.id))return;const f=1.2+Math.random()*3,p=u*2.399963%(Math.PI*2),g=(Math.random()-.5)*.4;this._solarAsteroids.set(d.id,{r:f,angle:p,y:g}),u++})}this.animateLayoutTo(e)}animateLayoutTo(e){const i=performance.now(),r=new Map;this.nodeObjects.forEach(o=>{r.set(o.userData.atomId,o.position.clone())});const s=()=>{if(this.disposed)return;const o=Math.min((performance.now()-i)/850,1),a=o<.5?2*o*o:-1+(4-2*o)*o;this.nodeObjects.forEach(l=>{const c=this.atomsById.get(l.userData.atomId);if(!c)return;const u=r.get(c.id)??l.position,d=e==="solar"?this.solarPos(c):e==="force"?this.forcePos(c):this.vertexForRegion(c,l.userData.region);l.position.lerpVectors(u,d,a);const f=this.haloMap.get(c.id);f&&f.position.copy(l.position);const p=this.labelMap.get(c.id);p&&(p.position.copy(l.position),p.position.y+=.02)}),o<1&&requestAnimationFrame(s)};requestAnimationFrame(s)}solarPos(e){if(e.id===this._solarSunId)return new L(0,0,0);const t=this._solarPlanets.get(e.id);if(t){const s=Math.sin(t.tilt)*t.orbitR*.3;return new L(Math.cos(t.angle)*t.orbitR,s,Math.sin(t.angle)*t.orbitR)}const i=this._solarMoons.get(e.id);if(i){const s=this.nodeMap.get(i.planetId);if(s)return new L(s.position.x+Math.cos(i.moonAngle)*i.moonR,s.position.y+Math.sin(i.moonAngle*.7)*i.moonR*.4,s.position.z+Math.sin(i.moonAngle)*i.moonR)}const r=this._solarAsteroids.get(e.id);return r?new L(Math.cos(r.angle)*r.r,r.y,Math.sin(r.angle)*r.r):new L(1.5,0,0)}forcePos(e){const t=this._forceIndex.get(e.id);if(t!==void 0)return new L(this._forcePosArr[t*3],this._forcePosArr[t*3+1],this._forcePosArr[t*3+2]);const i=this.nodeMap.get(e.id);return i?i.position.clone():new L}forceItersPerSlice(e){return e<=500?40:e<=1e3?15:e<=1500?8:3}computeForceLayout(){this.cancelForceSim();const e=[...this.atomsById.keys()].slice(0,e_),t=e.length;this._forceIds=e,this._forceIndex=new Map(e.map((r,s)=>[r,s])),this._forcePosArr=new Float32Array(t*3),this._forceVelArr=new Float32Array(t*3),this._forceIterations=0;for(let r=0;r<t;r++){const s=e[r],o=this.forcePositions.get(s);if(o)this._forcePosArr[r*3]=o.x,this._forcePosArr[r*3+1]=o.y,this._forcePosArr[r*3+2]=o.z;else{const l=this.nodeMap.get(s);l&&(this._forcePosArr[r*3]=l.position.x,this._forcePosArr[r*3+1]=l.position.y,this._forcePosArr[r*3+2]=l.position.z)}const a=Mm(s);this._forcePosArr[r*3]+=(a%1e3/1e3-.5)*.02,this._forcePosArr[r*3+1]+=((a>>>8)%1e3/1e3-.5)*.02,this._forcePosArr[r*3+2]+=((a>>>16)%1e3/1e3-.5)*.02}this._forceLinkPairs=[];const i=this.links.slice().sort((r,s)=>s.strength-r.strength).slice(0,t_);for(const r of i){const s=this._forceIndex.get(r.a),o=this._forceIndex.get(r.b);s!==void 0&&o!==void 0&&s!==o&&this._forceLinkPairs.push({a:s,b:o,strength:r.strength||0})}if(t<=1){t===1&&(this._forcePosArr[0]=0,this._forcePosArr[1]=0,this._forcePosArr[2]=0),this.finalizeForceLayout();return}this.scheduleForceSlice()}scheduleForceSlice(){this._forceSimHandle=setTimeout(()=>{this._forceSimHandle=null,!this.disposed&&this.forceSimSlice()},0)}forceSimSlice(){const e=this._forceIds.length;if(e<=1){this.finalizeForceLayout();return}const t=this._forcePosArr,i=this._forceVelArr,r=this.forceItersPerSlice(e),s=n_*n_,o=QC;for(let a=0;a<r;a++){this._forceIterations++;for(let u=0;u<e;u++){const d=u*3,f=u*3+1,p=u*3+2,g=t[d],y=t[f],m=t[p];let h=0,_=0,v=0;for(let x=u+1;x<e;x++){const R=x*3,A=x*3+1,w=x*3+2,b=g-t[R],T=y-t[A],M=m-t[w],I=b*b+T*T+M*M;if(I>=s||I<o)continue;const V=Math.sqrt(I),B=ZC/I,j=b/V,Y=T/V,W=M/V;h+=j*B,_+=Y*B,v+=W*B,i[R]-=j*B,i[A]-=Y*B,i[w]-=W*B}i[d]+=h,i[f]+=_,i[p]+=v}for(const u of this._forceLinkPairs){const d=u.a*3,f=u.a*3+1,p=u.a*3+2,g=u.b*3,y=u.b*3+1,m=u.b*3+2,h=t[g]-t[d],_=t[y]-t[f],v=t[m]-t[p],x=Math.sqrt(h*h+_*_+v*v)||o,R=x-qC,A=$C*R*(u.strength||1),w=h/x,b=_/x,T=v/x;i[d]+=w*A,i[f]+=b*A,i[p]+=T*A,i[g]-=w*A,i[y]-=b*A,i[m]-=T*A}let l=0;for(let u=0;u<e;u++){const d=u*3,f=u*3+1,p=u*3+2;i[d]-=t[d]*od,i[f]-=t[f]*od,i[p]-=t[p]*od,i[d]*=sd,i[f]*=sd,i[p]*=sd,t[d]+=i[d],t[f]+=i[f],t[p]+=i[p],l+=Math.abs(i[d])+Math.abs(i[f])+Math.abs(i[p])}if(l/(e*3)<JC||this._forceIterations>=KC){this.finalizeForceLayout();return}}this.scheduleForceSlice()}finalizeForceLayout(){this.cancelForceSim(),this.forcePositions=new Map;for(let e=0;e<this._forceIds.length;e++)this.forcePositions.set(this._forceIds[e],new L(this._forcePosArr[e*3],this._forcePosArr[e*3+1],this._forcePosArr[e*3+2]));this.layoutMode==="force"&&this.animateLayoutTo("force")}cancelForceSim(){this._forceSimHandle!==null&&(clearTimeout(this._forceSimHandle),this._forceSimHandle=null)}setTheme(e){if(this.palette=e==="light"?Jg:Qg,this.particleColorAttr){const t=new Pe;this.particleColorIndex.forEach((i,r)=>{t.setHex(this.palette.particles[i]),this.particleColorAttr.setXYZ(r,t.r,t.g,t.b)}),this.particleColorAttr.needsUpdate=!0}this.clearHoverSilent(),this.applyRegionColors(),this.edgeGroup.children.forEach(t=>{t.material.color.setHex(this.palette.edgeRest)})}clearHoverSilent(){this.hover=null}addListeners(){const e=this.renderer.domElement;e.addEventListener("pointermove",this.onPointerMove),e.addEventListener("pointerdown",this.onPointerDown),window.addEventListener("pointerup",this.onPointerUp),e.addEventListener("wheel",this.onWheel,{passive:!1}),e.addEventListener("click",this.onClick),this.container.addEventListener("keydown",this.onKeydown)}removeListeners(){var t;const e=(t=this.renderer)==null?void 0:t.domElement;e&&(e.removeEventListener("pointermove",this.onPointerMove),e.removeEventListener("pointerdown",this.onPointerDown),e.removeEventListener("wheel",this.onWheel),e.removeEventListener("click",this.onClick)),window.removeEventListener("pointerup",this.onPointerUp),this.container.removeEventListener("keydown",this.onKeydown)}updateNdc(e){const t=this.renderer.domElement.getBoundingClientRect();this.pointerNdc.set((e.clientX-t.left)/t.width*2-1,-((e.clientY-t.top)/t.height)*2+1),this.parallaxTarget.set(this.pointerNdc.x*.07,this.pointerNdc.y*.045)}resize(){const e=this.container.getBoundingClientRect();!e.width||!e.height||(this.camera.aspect=e.width/e.height,this.camera.updateProjectionMatrix(),this.renderer.setSize(e.width,e.height,!1))}dispose(){var e,t,i;this.disposed=!0,cancelAnimationFrame(this.raf),this.cancelForceSim(),(e=this.resizeObserver)==null||e.disconnect(),this.removeListeners(),this.haloTexture.dispose(),this.sphereGeo.dispose(),this.brainGroup.traverse(r=>{var l,c,u,d,f,p,g,y;const s=r;if(s.isMesh){(c=(l=s.geometry)==null?void 0:l.dispose)==null||c.call(l);const m=s.material;Array.isArray(m)?m.forEach(h=>h.dispose()):(u=m==null?void 0:m.dispose)==null||u.call(m)}const o=r;o.isPoints&&((f=(d=o.geometry)==null?void 0:d.dispose)==null||f.call(d),(g=(p=o.material)==null?void 0:p.dispose)==null||g.call(p));const a=r;if(a.isSprite){const m=a.material;(y=m.map)==null||y.dispose(),m.dispose()}}),(t=this.renderer)==null||t.dispose(),(i=this.renderer)==null||i.domElement.remove()}}const tb=Ne.forwardRef(function({atoms:e,renderCap:t,layout:i,atomTotal:r,onSelect:s,onHover:o,caption:a},l){const c=Ne.useRef(null),u=Ne.useRef(null),[d,f]=Ne.useState(null),[p,g]=Ne.useState(!0),[y,m]=Ne.useState(!1);Ne.useImperativeHandle(l,()=>({focusAtom:_=>{u.current&&(u.current.focusAtom(_),u.current.zoomToAtom(_))},randomConnection:()=>{u.current&&u.current.randomConnection()}}));const h=Ne.useCallback((_,v,x)=>{if(!_){f(null);return}f({atom:_,x:v,y:x})},[]);return Ne.useEffect(()=>{if(!c.current)return;const _=new eb(c.current,{theme:"dark",reducedMotion:window.matchMedia("(prefers-reduced-motion: reduce)").matches,onHoverAtom:(v,x,R)=>{o(v),h(v,x,R)},onSelectAtom:v=>{s(v)},onClearHover:()=>{o(null),f(null)},onZoomOut:()=>{},onReady:()=>m(!0),onError:v=>console.error("[BrainView]",v)});return u.current=_,()=>{_.dispose(),u.current=null}},[]),Ne.useEffect(()=>{if(!u.current||!y)return;const _=e.slice(0,t),v=nb(_);u.current.setData(_,v),g(_.length===0)},[e,t,y]),Ne.useEffect(()=>{u.current&&u.current.setLayout(i)},[i]),H.jsxs("div",{id:"brain-stage",className:"brain-stage",tabIndex:0,role:"application","aria-label":"Interactive three-dimensional memory lattice.",children:[p&&H.jsx("div",{id:"lattice-fallback",className:"lattice-fallback","aria-hidden":"true",children:H.jsxs("svg",{className:"fallback-svg",viewBox:"0 0 800 400",preserveAspectRatio:"xMidYMid slice",children:[H.jsxs("radialGradient",{id:"bgGlow",cx:"50%",cy:"50%",r:"50%",children:[H.jsx("stop",{offset:"0%",stopColor:"#a855f7",stopOpacity:.12}),H.jsx("stop",{offset:"100%",stopColor:"#050508",stopOpacity:0})]}),H.jsx("rect",{width:"800",height:"400",fill:"url(#bgGlow)"}),[[400,200,200,100],[400,200,600,100],[400,200,600,300],[400,200,200,300],[200,100,600,100],[600,100,600,300],[600,300,200,300],[200,300,200,100],[400,200,400,60],[400,200,400,340],[400,200,100,200],[400,200,700,200]].map(([_,v,x,R],A)=>H.jsx("line",{className:"fallback-edge",x1:_,y1:v,x2:x,y2:R},A)),[[100,60],[700,60],[700,340],[100,340],[400,60],[400,340],[100,200],[700,200]].map(([_,v],x)=>H.jsx("circle",{className:"fallback-node",cx:_,cy:v,r:"5"},x)),[[200,100],[600,100],[600,300],[200,300]].map(([_,v],x)=>H.jsx("circle",{className:"fallback-node-white",cx:_,cy:v,r:"7"},x)),H.jsxs("filter",{id:"centreGlow",x:"-80%",y:"-80%",width:"260%",height:"260%",children:[H.jsx("feGaussianBlur",{stdDeviation:"5",result:"b"}),H.jsxs("feMerge",{children:[H.jsx("feMergeNode",{in:"b"}),H.jsx("feMergeNode",{in:"SourceGraphic"})]})]}),H.jsx("circle",{cx:"400",cy:"200",r:"12",fill:"#a855f7",filter:"url(#centreGlow)"})]})}),H.jsx("div",{id:"brain-canvas",ref:c,"aria-label":"3D memory graph",style:{width:"100%",height:"100%"}}),H.jsxs("div",{className:"brain-overlay","aria-hidden":"true",children:[H.jsx("span",{children:"DRAG / ORBIT"}),H.jsx("span",{children:"SCROLL / ZOOM"})]}),d&&H.jsxs("div",{id:"node-tooltip",className:"node-tooltip",role:"status","aria-live":"polite",style:{left:d.x+12,top:d.y+12},children:[H.jsx("strong",{children:d.atom.title}),H.jsxs("span",{children:[d.atom.tags.length," tags · ",d.atom.tier]})]}),H.jsxs("div",{className:"brain-caption",children:[H.jsx("span",{className:"pulse-ring","aria-hidden":"true"}),H.jsx("span",{id:"brain-caption",children:a})]})]})});function nb(n){const e=new Map;n.forEach(r=>{r.tags.forEach(s=>{const o=e.get(s)??[];o.push(r.id),e.set(s,o)})});const t=new Set,i=[];return e.forEach(r=>{for(let s=0;s<r.length;s++)for(let o=s+1;o<r.length;o++){const a=r[s]<r[o]?`${r[s]}:${r[o]}`:`${r[o]}:${r[s]}`;t.has(a)||(t.add(a),i.push({a:r[s],b:r[o],strength:1}))}}),i}function ib({layout:n,maxMode:e,atomTotal:t,atomsLoaded:i,onLayoutChange:r,onMaxMode:s,onSince:o,onSelectAtom:a,onRandom:l}){const[c,u]=Ne.useState(""),[d,f]=Ne.useState([]),[p,g]=Ne.useState(!1),[y,m]=Ne.useState(100),[h,_]=Ne.useState(!1),v=Ne.useRef(null),x=Ne.useRef(null);Ne.useEffect(()=>{var b;if(!c.trim()){f([]),g(!1);return}(b=x.current)==null||b.abort();const w=new AbortController;return x.current=w,jS(c,w.signal).then(T=>{f(T),g(T.length>0)}).catch(()=>{}),()=>w.abort()},[c]),Ne.useEffect(()=>{if(v.current&&clearInterval(v.current),!!h)return v.current=setInterval(()=>{m(w=>{const b=w>=100?0:w+1;return o(b/100),b})},80),()=>{v.current&&clearInterval(v.current)}},[h,o]);const R=w=>{m(w),o(w/100)},A=y>=100?"all time":`${y}%`;return H.jsxs("section",{className:"command-strip","aria-label":"Brain controls",children:[H.jsxs("label",{className:"search-control",htmlFor:"brain-search",children:[H.jsx("span",{"aria-hidden":"true",children:"⌕"}),H.jsx("input",{id:"brain-search",type:"search",autoComplete:"off",placeholder:"Search your memory",value:c,onChange:w=>u(w.target.value),onBlur:()=>setTimeout(()=>g(!1),150)}),H.jsx("kbd",{children:"⌘ K"})]}),p&&H.jsx("div",{id:"search-dropdown",className:"search-dropdown",role:"listbox","aria-label":"Search results",children:d.slice(0,6).map(w=>H.jsxs("button",{type:"button",className:"search-result",role:"option",onMouseDown:()=>{a(w),g(!1),u("")},children:[H.jsx("strong",{children:w.title}),H.jsx("span",{children:w.summary})]},w.id))}),H.jsxs("div",{className:"timeline-control",children:[H.jsx("button",{id:"timeline-play",className:"icon-button",type:"button","aria-label":h?"Pause timeline":"Play timeline","aria-pressed":h,onClick:()=>_(w=>!w),children:h?"⏸":"▶"}),H.jsx("label",{htmlFor:"timeline-range",children:"Memory horizon"}),H.jsx("input",{id:"timeline-range",type:"range",min:"0",max:"100",value:y,onChange:w=>R(Number(w.target.value))}),H.jsx("output",{id:"timeline-output",children:A})]}),H.jsx("button",{className:`quiet-button layout-toggle${n==="regions"?" is-active":""}`,type:"button","aria-pressed":n==="regions","aria-label":"Switch to brain regions layout",onClick:()=>r("regions"),children:"brain"}),H.jsx("button",{className:`quiet-button layout-toggle${n==="solar"?" is-active":""}`,type:"button","aria-pressed":n==="solar","aria-label":"Switch to solar system layout",onClick:()=>r("solar"),children:"solar"}),H.jsxs("button",{id:"max-toggle",className:`quiet-button max-toggle${e?" is-active":""}`,type:"button","aria-pressed":e,"aria-label":e?"Exit max mode and reload the standard memory subset":"Load up to 10,000 memories via tiered graph",title:"Hover: ghost preview · Click: load up to 10,000 memories (hot → warm → cold)",onClick:()=>s(!e),onMouseEnter:()=>{},children:["max",e&&t>i&&H.jsxs("small",{style:{marginLeft:4,opacity:.7},children:[i,"/",t]})]}),H.jsx("button",{id:"random-pick",className:"quiet-button",type:"button","aria-label":"Jump to a random memory",title:"Random: zoom to a random memory and highlight its connections",onClick:l,children:"random"})]})}function rb({live:n,onLiveToggle:e}){const[t,i]=Ne.useState([]),r=Ne.useRef(null);return Ne.useEffect(()=>{if(!n)return;let s=!0;return(async()=>{var a;for(;s;){try{(a=r.current)==null||a.abort();const l=new AbortController;r.current=l;const c=await XS(l.signal);s&&i(c.slice(0,30))}catch{}await new Promise(l=>setTimeout(l,4e3))}})(),()=>{var a;s=!1,(a=r.current)==null||a.abort()}},[n]),H.jsxs("aside",{className:"panel stream-panel","aria-labelledby":"stream-heading",children:[H.jsxs("div",{className:"panel-heading",children:[H.jsx("p",{className:"eyebrow",children:"Pulse"}),H.jsx("h2",{id:"stream-heading",children:"Memory stream"}),H.jsx("button",{id:"stream-toggle",className:"quiet-button",type:"button","aria-pressed":n,onClick:()=>e(!n),children:n?"live":"paused"})]}),H.jsx("div",{id:"activity-stream",className:"activity-stream","aria-live":"polite",children:t.length===0?H.jsx("p",{className:"empty-state",children:"Listening for local activity…"}):t.map(s=>H.jsxs("div",{className:"activity-item",children:[H.jsx("span",{className:"activity-event",children:s.event}),s.title&&H.jsx("span",{className:"activity-title",children:s.title}),s.source&&H.jsx("span",{className:"activity-source",children:s.source})]},s.id))})]})}function sb(n,e){return e.filter(t=>t.id===n.id?!1:n.tags.filter(r=>t.tags.includes(r)).length>0||n.source&&n.source===t.source).length}function ad(n){return`#${(n>>>0).toString(16).padStart(6,"0")}`}function ld(n,e,t){const i=n>>16&255,r=n>>8&255,s=n&255,o=e>>16&255,a=e>>8&255,l=e&255;return Math.round(i+(o-i)*t)<<16|Math.round(r+(a-r)*t)<<8|Math.round(s+(l-s)*t)}function ob(n){let e=0;for(let t=0;t<n.length;t++)e=(e<<5)-e+n.charCodeAt(t)|0;return(e>>>0)/4294967295}const wl=[4988309,5972406,7153881,8141549,9133302,10980346,12891645];function ab(n){const t=ob(n)*(wl.length-1),i=Math.floor(t),r=t-i,s=wl[i],o=wl[Math.min(i+1,wl.length-1)],a=ld(s,o,r),l=ld(a,3018853,.45),c=ld(a,16777215,.28);return`linear-gradient(135deg, ${ad(l)}, ${ad(a)} 55%, ${ad(c)})`}function lb({atom:n,allAtoms:e}){const[t,i]=Ne.useState(null);Ne.useEffect(()=>{if(!n){i(null);return}i(n),n.lean&&KS(n.id).then(s=>{s&&i(s)})},[n]);const r=t??n;return H.jsxs("section",{className:"panel inspector-panel","aria-labelledby":"inspector-heading",children:[H.jsxs("div",{className:"panel-heading",children:[H.jsx("p",{className:"eyebrow",children:"Focus"}),H.jsx("h2",{id:"inspector-heading",children:"Node inspector"})]}),H.jsx("div",{id:"node-inspector",className:"node-inspector","aria-live":"polite",children:r?H.jsxs(H.Fragment,{children:[H.jsx("h3",{className:"inspector-title",children:r.title}),H.jsx("p",{className:"inspector-summary",children:r.summary}),H.jsx("div",{className:"node-metrics",children:[["weight",r.score.toFixed(2)],["recency",r.last_accessed_at||r.indexed_at||"unknown"],["relations",sb(r,e)],["tier",r.tier]].map(([s,o])=>H.jsxs("span",{className:"metric",children:[s," ",H.jsx("b",{children:o})]},s))}),r.tags.length>0&&H.jsx("div",{className:"inspector-tags",children:r.tags.map(s=>H.jsxs("span",{className:"tag-chip",style:{background:ab(s)},children:["#",s]},s))}),r.file&&H.jsx("button",{className:"open-button",type:"button",onClick:()=>qS(r.file),children:"Open source file ↗"})]}):H.jsx("p",{className:"empty-state",children:"Hover or select a memory node to reveal its place in the lattice."})})]})}function cb(){const[n,e]=Ne.useState(""),[t,i]=Ne.useState("Answers stay on your local daemon."),[r,s]=Ne.useState(!1),o=Ne.useRef(null),a=async l=>{var u;if(l.preventDefault(),!n.trim()||r)return;(u=o.current)==null||u.abort();const c=new AbortController;o.current=c,s(!0),i("Thinking…");try{const d=await YS(n.trim(),c.signal);i(d)}catch(d){d.name!=="AbortError"&&i("The daemon could not answer that question.")}finally{s(!1)}};return H.jsxs("section",{className:"panel ask-panel","aria-labelledby":"ask-heading",children:[H.jsxs("div",{className:"panel-heading",children:[H.jsx("p",{className:"eyebrow",children:"Synthesize"}),H.jsx("h2",{id:"ask-heading",children:"Ask the brain"})]}),H.jsxs("form",{id:"ask-form",className:"ask-form",onSubmit:a,children:[H.jsx("label",{className:"sr-only",htmlFor:"ask-input",children:"Ask a question about your memory"}),H.jsx("textarea",{id:"ask-input",rows:3,placeholder:"What connects these memories?",value:n,onChange:l=>e(l.target.value),disabled:r}),H.jsxs("button",{type:"submit",className:"primary-button",disabled:r||!n.trim(),children:["Ask ",H.jsx("span",{"aria-hidden":"true",children:"↗"})]})]}),H.jsx("div",{id:"ask-output",className:"ask-output","aria-live":"polite",children:t})]})}function ub({atoms:n,atomTotal:e}){const t=n.filter(o=>o.tier==="hot").length,i=n.filter(o=>o.tier==="warm").length,r=n.filter(o=>o.tier==="cold").length,s=n.filter(o=>o.score>=.8).length;return H.jsxs("section",{className:"panel signal-panel","aria-labelledby":"signal-heading",children:[H.jsxs("div",{className:"panel-heading",children:[H.jsx("p",{className:"eyebrow",children:"Signal"}),H.jsx("h2",{id:"signal-heading",children:"Brain state"})]}),H.jsxs("dl",{className:"stats",children:[H.jsxs("div",{children:[H.jsx("dt",{children:"memories"}),H.jsx("dd",{id:"stat-atoms",children:e>n.length?`${n.length} / ${e}`:n.length||"—"})]}),H.jsxs("div",{children:[H.jsx("dt",{children:"hot / warm / cold"}),H.jsx("dd",{id:"stat-tiers",children:n.length?`${t} / ${i} / ${r}`:"—"})]}),H.jsxs("div",{children:[H.jsx("dt",{children:"trusted"}),H.jsx("dd",{id:"stat-trusted",children:n.length?s:"—"})]})]})]})}const i_=450;function cd(n){const e=Date.parse(n.indexed_at||n.last_accessed_at||"");return Number.isFinite(e)?e:0}function db(){const[n,e]=Ne.useReducer(kS,BS),[t,i]=Ne.useState(null),[r,s]=Ne.useState(!0),o=Ne.useRef(null),a=Ne.useRef(null),l=Ne.useCallback(()=>{if(n.since<=0||!n.atoms.length)return n.atoms;const _=n.since,v=n.atoms.map(cd).filter(Boolean),x=Math.min(...v,Date.now()),R=x+(Date.now()-x)*(1-_);return n.atoms.filter(A=>!cd(A)||cd(A)>=R)},[n.atoms,n.since]),c=Ne.useCallback(async()=>{var v;(v=o.current)==null||v.abort();const _=new AbortController;o.current=_;try{let x;n.maxMode?x=await WS(_.signal):x=await GS(i_,_.signal),e({type:"SET_ATOMS",atoms:x,total:x.length})}catch{}},[n.maxMode]);Ne.useEffect(()=>{let _=!0;return(async()=>{var x;for(;_;){try{(x=o.current)==null||x.abort();const R=new AbortController;o.current=R;const A=await VS(R.signal);e({type:"SET_DAEMON",ok:A.ok,version:A.version})}catch{}await new Promise(R=>setTimeout(R,8e3))}})(),()=>{var x;_=!1,(x=o.current)==null||x.abort()}},[]),Ne.useEffect(()=>{c()},[n.maxMode]);const u=_=>{e({type:"SET_LAYOUT",layout:_})},d=async _=>{e({type:"SET_MAX_MODE",maxMode:_}),localStorage.setItem("kurultai-max-mode",_?"1":"0")},f=_=>{e({type:"SET_SINCE",since:_})},p=_=>{var v;i(_),(v=a.current)==null||v.focusAtom(_)},g=()=>{var _;(_=a.current)==null||_.randomConnection()},y=l(),m=n.maxMode?2500:i_,h=(()=>{const _=Math.min(y.length,m),v=Math.max(n.atomTotal,n.atoms.length),x=n.maxMode?"max":"standard";return n.maxMode&&y.length>_?`${_} shown · ${y.length} loaded (cap ${m}) · ${x}`:v>y.length&&!n.maxMode?`${y.length} of ${v} memories · ${x} · hover max for ghost preview`:`${_} memories · ${x} · hover to trace connections`})();return H.jsxs(zS.Provider,{value:{state:n,dispatch:e},children:[H.jsx("a",{className:"skip-link",href:"#workspace",children:"Skip to workspace"}),H.jsx(ZS,{daemonOk:n.daemonOk,daemonVersion:n.daemonVersion}),H.jsxs("main",{id:"workspace",children:[H.jsx("section",{id:"brain",className:"brain-hero","aria-label":"Brain visualization",children:H.jsx(tb,{ref:a,atoms:y,renderCap:m,layout:n.layout,atomTotal:n.atomTotal,onSelect:i,onHover:_=>{_&&i(_)},caption:h})}),H.jsx(ib,{layout:n.layout,maxMode:n.maxMode,atomTotal:n.atomTotal,atomsLoaded:n.atoms.length,onLayoutChange:u,onMaxMode:d,onSince:f,onSelectAtom:p,onRandom:g}),H.jsxs("section",{className:"dashboard-grid","aria-label":"Brain dashboard",children:[H.jsx(rb,{live:r,onLiveToggle:s}),H.jsx(lb,{atom:t,allAtoms:y}),H.jsx(cb,{}),H.jsx(ub,{atoms:y,atomTotal:n.atomTotal})]})]}),H.jsx("footer",{children:"Kurultai runs locally. Your knowledge remains yours."})]})}const sy=document.getElementById("root");if(!sy)throw new Error("No #root element found");gv(sy).render(H.jsx(Ne.StrictMode,{children:H.jsx(db,{})}));
