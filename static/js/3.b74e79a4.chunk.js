(this["webpackJsonpaleo-website"]=this["webpackJsonpaleo-website"]||[]).push([[3],{198:function(n,r,t){"use strict";(function(n,e){t.d(r,"a",(function(){return E})),t.d(r,"b",(function(){return C})),t.d(r,"c",(function(){return q})),t.d(r,"d",(function(){return D})),t.d(r,"F",(function(){return F})),t.d(r,"D",(function(){return P})),t.d(r,"C",(function(){return U})),t.d(r,"s",(function(){return B})),t.d(r,"h",(function(){return I})),t.d(r,"r",(function(){return R})),t.d(r,"y",(function(){return z})),t.d(r,"w",(function(){return G})),t.d(r,"q",(function(){return J})),t.d(r,"z",(function(){return S})),t.d(r,"l",(function(){return V})),t.d(r,"g",(function(){return H})),t.d(r,"m",(function(){return M})),t.d(r,"o",(function(){return K})),t.d(r,"f",(function(){return L})),t.d(r,"t",(function(){return N})),t.d(r,"x",(function(){return Q})),t.d(r,"i",(function(){return W})),t.d(r,"j",(function(){return X})),t.d(r,"A",(function(){return Y})),t.d(r,"e",(function(){return Z})),t.d(r,"n",(function(){return $})),t.d(r,"u",(function(){return nn})),t.d(r,"k",(function(){return rn})),t.d(r,"p",(function(){return tn})),t.d(r,"v",(function(){return en})),t.d(r,"G",(function(){return un})),t.d(r,"E",(function(){return on})),t.d(r,"B",(function(){return cn}));var u=t(201),i=t(202),o=t(199),c=new("undefined"===typeof TextDecoder?(0,n.require)("util").TextDecoder:TextDecoder)("utf-8",{ignoreBOM:!0,fatal:!0});c.decode();var f=null;function a(){return null!==f&&f.buffer===o.o.buffer||(f=new Uint8Array(o.o.buffer)),f}function _(n,r){return c.decode(a().subarray(n,n+r))}var d=new Array(32).fill(void 0);d.push(void 0,null,!0,!1);var l=d.length;function s(n){l===d.length&&d.push(d.length+1);var r=l;return l=d[r],d[r]=n,r}function y(n){return d[n]}function v(n){var r=y(n);return function(n){n<36||(d[n]=l,l=n)}(n),r}var b=0,p=new("undefined"===typeof TextEncoder?(0,n.require)("util").TextEncoder:TextEncoder)("utf-8"),w="function"===typeof p.encodeInto?function(n,r){return p.encodeInto(n,r)}:function(n,r){var t=p.encode(n);return r.set(t),{read:n.length,written:t.length}};function g(n,r,t){if(void 0===t){var e=p.encode(n),u=r(e.length);return a().subarray(u,u+e.length).set(e),b=e.length,u}for(var i=n.length,o=r(i),c=a(),f=0;f<i;f++){var _=n.charCodeAt(f);if(_>127)break;c[o+f]=_}if(f!==i){0!==f&&(n=n.slice(f)),o=t(o,i,i=f+3*n.length);var d=a().subarray(o+f,o+i);f+=w(n,d).written}return b=f,o}var h=null;function k(){return null!==h&&h.buffer===o.o.buffer||(h=new Int32Array(o.o.buffer)),h}var m=null;function j(n,r){for(var t=(null!==m&&m.buffer===o.o.buffer||(m=new Uint32Array(o.o.buffer)),m).subarray(n/4,n/4+r),e=[],u=0;u<t.length;u++)e.push(v(t[u]));return e}var O=new Uint32Array(2),x=new BigUint64Array(O.buffer);function A(n,r){try{return n.apply(this,r)}catch(t){o.f(s(t))}}function T(n,r){return a().subarray(n/1,n/1+r)}var E=function(){function n(){Object(u.a)(this,n);var r=o.k();return n.__wrap(r)}return Object(i.a)(n,[{key:"__destroy_into_raw",value:function(){var n=this.ptr;return this.ptr=0,n}},{key:"free",value:function(){var n=this.__destroy_into_raw();o.a(n)}},{key:"to_private_key",value:function(){try{var n=o.e(-16);o.m(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}},{key:"to_view_key",value:function(){try{var n=o.e(-16);o.n(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}},{key:"to_address",value:function(){try{var n=o.e(-16);o.l(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}}],[{key:"__wrap",value:function(r){var t=Object.create(n.prototype);return t.ptr=r,t}},{key:"from_private_key",value:function(r){var t=g(r,o.h,o.i),e=b,u=o.j(t,e);return n.__wrap(u)}}]),n}(),C=function(){function n(){Object(u.a)(this,n)}return Object(i.a)(n,[{key:"__destroy_into_raw",value:function(){var n=this.ptr;return this.ptr=0,n}},{key:"free",value:function(){var n=this.__destroy_into_raw();o.b(n)}},{key:"owner",value:function(){try{var n=o.e(-16);o.t(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}},{key:"is_dummy",value:function(){return 0!==o.s(this.ptr)}},{key:"payload",value:function(){try{var n=o.e(-16);o.u(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}},{key:"program_id",value:function(){try{var n=o.e(-16);o.v(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}},{key:"serial_number_nonce",value:function(){try{var n=o.e(-16);o.w(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}},{key:"commitment",value:function(){try{var n=o.e(-16);o.p(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}},{key:"commitment_randomness",value:function(){try{var n=o.e(-16);o.q(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}},{key:"value",value:function(){try{var n=o.e(-16);o.y(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return O[0]=r,O[1]=t,x[0]}finally{o.e(16)}}},{key:"to_string",value:function(){try{var n=o.e(-16);o.x(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}}],[{key:"__wrap",value:function(r){var t=Object.create(n.prototype);return t.ptr=r,t}},{key:"from_string",value:function(r){var t=g(r,o.h,o.i),e=b,u=o.r(t,e);return n.__wrap(u)}}]),n}(),q=function(){function n(){Object(u.a)(this,n)}return Object(i.a)(n,[{key:"__destroy_into_raw",value:function(){var n=this.ptr;return this.ptr=0,n}},{key:"free",value:function(){var n=this.__destroy_into_raw();o.c(n)}},{key:"decrypt",value:function(n){var r=g(n,o.h,o.i),t=b,e=o.z(this.ptr,r,t);return C.__wrap(e)}},{key:"to_string",value:function(){try{var n=o.e(-16);o.C(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}}],[{key:"__wrap",value:function(r){var t=Object.create(n.prototype);return t.ptr=r,t}},{key:"new",value:function(r){var t=g(r,o.h,o.i),e=b,u=o.B(t,e);return n.__wrap(u)}},{key:"encrypt",value:function(r){!function(n,r){if(!(n instanceof r))throw new Error("expected instance of ".concat(r.name));n.ptr}(r,C);var t=r.ptr;r.ptr=0;var e=o.A(t);return n.__wrap(e)}}]),n}(),D=function(){function n(){Object(u.a)(this,n)}return Object(i.a)(n,[{key:"__destroy_into_raw",value:function(){var n=this.ptr;return this.ptr=0,n}},{key:"free",value:function(){var n=this.__destroy_into_raw();o.d(n)}},{key:"to_decrypted_records",value:function(n){try{var r=o.e(-16),t=g(n,o.h,o.i),e=b;o.E(r,this.ptr,t,e);var u=k()[r/4+0],i=k()[r/4+1],c=j(u,i).slice();return o.g(u,4*i),c}finally{o.e(16)}}},{key:"to_string",value:function(){try{var n=o.e(-16);o.F(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}},{key:"transitions",value:function(){try{var n=o.e(-16);o.H(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}},{key:"transaction_id",value:function(){try{var n=o.e(-16);o.G(n,this.ptr);var r=k()[n/4+0],t=k()[n/4+1];return _(r,t)}finally{o.e(16),o.g(r,t)}}}],[{key:"__wrap",value:function(r){var t=Object.create(n.prototype);return t.ptr=r,t}},{key:"from_string",value:function(r){var t=g(r,o.h,o.i),e=b,u=o.D(t,e);return n.__wrap(u)}}]),n}();function F(n,r){return s(_(n,r))}function P(n){v(n)}function U(n){return s(y(n))}function B(){return A((function(n,r,t){y(n).randomFillSync(T(r,t))}),arguments)}function I(){return A((function(n,r){y(n).getRandomValues(y(r))}),arguments)}function R(n){return s(y(n).process)}function z(n){var r=y(n);return"object"===typeof r&&null!==r}function G(n){return s(y(n).versions)}function J(n){return s(y(n).node)}function S(n){return"string"===typeof y(n)}function V(){return A((function(n,r){return s(t(203)(_(n,r)))}),arguments)}function H(n){return s(y(n).crypto)}function M(n){return s(y(n).msCrypto)}function K(n,r){return s(new Function(_(n,r)))}function L(){return A((function(n,r){return s(y(n).call(y(r)))}),arguments)}function N(){return A((function(){return s(self.self)}),arguments)}function Q(){return A((function(){return s(window.window)}),arguments)}function W(){return A((function(){return s(globalThis.globalThis)}),arguments)}function X(){return A((function(){return s(e.global)}),arguments)}function Y(n){return void 0===y(n)}function Z(n){return s(y(n).buffer)}function $(n){return s(new Uint8Array(y(n)))}function nn(n,r,t){y(n).set(y(r),t>>>0)}function rn(n){return y(n).length}function tn(n){return s(new Uint8Array(n>>>0))}function en(n,r,t){return s(y(n).subarray(r>>>0,t>>>0))}function un(n,r){throw new Error(_(n,r))}function on(n){throw v(n)}function cn(){return s(o.o)}}).call(this,t(200)(n),t(86))},199:function(n,r,t){"use strict";var e=t.w[n.i];n.exports=e;t(198);e.I()},200:function(n,r){n.exports=function(n){if(!n.webpackPolyfill){var r=Object.create(n);r.children||(r.children=[]),Object.defineProperty(r,"loaded",{enumerable:!0,get:function(){return r.l}}),Object.defineProperty(r,"id",{enumerable:!0,get:function(){return r.i}}),Object.defineProperty(r,"exports",{enumerable:!0}),r.webpackPolyfill=1}return r}},201:function(n,r,t){"use strict";function e(n,r){if(!(n instanceof r))throw new TypeError("Cannot call a class as a function")}t.d(r,"a",(function(){return e}))},202:function(n,r,t){"use strict";function e(n,r){for(var t=0;t<r.length;t++){var e=r[t];e.enumerable=e.enumerable||!1,e.configurable=!0,"value"in e&&(e.writable=!0),Object.defineProperty(n,e.key,e)}}function u(n,r,t){return r&&e(n.prototype,r),t&&e(n,t),n}t.d(r,"a",(function(){return u}))},204:function(n,r,t){"use strict";t.r(r);var e=t(198);t.d(r,"Account",(function(){return e.a})),t.d(r,"Record",(function(){return e.b})),t.d(r,"RecordCiphertext",(function(){return e.c})),t.d(r,"Transaction",(function(){return e.d})),t.d(r,"__wbindgen_string_new",(function(){return e.F})),t.d(r,"__wbindgen_object_drop_ref",(function(){return e.D})),t.d(r,"__wbindgen_object_clone_ref",(function(){return e.C})),t.d(r,"__wbg_randomFillSync_64cc7d048f228ca8",(function(){return e.s})),t.d(r,"__wbg_getRandomValues_98117e9a7e993920",(function(){return e.h})),t.d(r,"__wbg_process_2f24d6544ea7b200",(function(){return e.r})),t.d(r,"__wbindgen_is_object",(function(){return e.y})),t.d(r,"__wbg_versions_6164651e75405d4a",(function(){return e.w})),t.d(r,"__wbg_node_4b517d861cbcb3bc",(function(){return e.q})),t.d(r,"__wbindgen_is_string",(function(){return e.z})),t.d(r,"__wbg_modulerequire_3440a4bcf44437db",(function(){return e.l})),t.d(r,"__wbg_crypto_98fc271021c7d2ad",(function(){return e.g})),t.d(r,"__wbg_msCrypto_a2cdb043d2bfe57f",(function(){return e.m})),t.d(r,"__wbg_newnoargs_be86524d73f67598",(function(){return e.o})),t.d(r,"__wbg_call_888d259a5fefc347",(function(){return e.f})),t.d(r,"__wbg_self_c6fbdfc2918d5e58",(function(){return e.t})),t.d(r,"__wbg_window_baec038b5ab35c54",(function(){return e.x})),t.d(r,"__wbg_globalThis_3f735a5746d41fbd",(function(){return e.i})),t.d(r,"__wbg_global_1bc0b39582740e95",(function(){return e.j})),t.d(r,"__wbindgen_is_undefined",(function(){return e.A})),t.d(r,"__wbg_buffer_397eaa4d72ee94dd",(function(){return e.e})),t.d(r,"__wbg_new_a7ce447f15ff496f",(function(){return e.n})),t.d(r,"__wbg_set_969ad0a60e51d320",(function(){return e.u})),t.d(r,"__wbg_length_1eb8fc608a0d4cdb",(function(){return e.k})),t.d(r,"__wbg_newwithlength_929232475839a482",(function(){return e.p})),t.d(r,"__wbg_subarray_8b658422a224f479",(function(){return e.v})),t.d(r,"__wbindgen_throw",(function(){return e.G})),t.d(r,"__wbindgen_rethrow",(function(){return e.E})),t.d(r,"__wbindgen_memory",(function(){return e.B}))}}]);
//# sourceMappingURL=3.b74e79a4.chunk.js.map