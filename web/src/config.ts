// SECCIÓN CONSTANTES MODIFICABLES
export const DRAWER_WIDTH = 260; // px
export const MINI_DRAWER_WIDTH = 90; // px

// Configuración del puerto para depuración
// Este puerto debe ser el mismo que el del servidor
const puertoDebug = '5000'

// FIN SECCIÓN CONSTANTES MODIFICABLES

// Variables de entorno
// Si se modifica este nombre cambiarlo en /scripts/web/build.sh
const APP = import.meta.env.VITE_CONTROLA_WEB_APP;

// Configuración para la comunicación con el API
const protocol = window.location.protocol;
const currentDomain = window.location.hostname;

const puerto = APP ? '' : `:${puertoDebug}`;
const parte = (APP ? `${APP}/` : '') + 'api';

export const URL_BASE_ROUTER = APP ? `/${APP}` : '';
export const URL_BASE_API = `${protocol}//${currentDomain}${puerto}/${parte}`;
