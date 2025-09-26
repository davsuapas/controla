import { AxiosRequestConfig } from 'axios';
import axios, { AxiosError } from 'axios';

import { UseNotifications } from '../hooks/useNotifications/useNotifications';
import { DialogHook } from '../hooks/useDialogs/useDialogs';
import { UseUsuarioLogeado } from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { logError } from '../error';

// Variables globales para el interceptor
let dialogo: DialogHook | null = null;
let notifica: UseNotifications | null = null;
let usrLogeado: UseUsuarioLogeado | null = null;

// Conecta el UI con el interceptor
export const configurarInterceptor = (
  dialogoParam: DialogHook,
  notificaParam: UseNotifications,
  usrLogeadoParam: UseUsuarioLogeado
) => {
  dialogo = dialogoParam;
  notifica = notificaParam;
  usrLogeado = usrLogeadoParam;
};

axios.defaults.timeout = 10000; //10 sg

const protocol = window.location.protocol;
const currentDomain = window.location.hostname;
axios.defaults.baseURL = `${protocol}//${currentDomain}:8080`;
axios.defaults.withCredentials = true; // Incluye cookies HttpOnly

// Error controlado
//
// Es un error que se maneja en el interceptor
export class NetErrorControlado {
  constructor(public readonly origen: AxiosError) {
  }
}

export interface ConfigRequest extends AxiosRequestConfig {
  manejarAuth?: boolean;
}

// Interceptor de response para manejo de errores
axios.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    if (!notifica || !dialogo) {
      // Si no hay sistema de notificaciones no se maneja el error
      // por el interceptor, sino por el usuario
      return Promise.reject(error);
    }

    if (!error.response) {
      // Error de red
      await notifica.show(
        'Error de conexión. Verifique su conexión a internet.');

      return Promise.reject(new NetErrorControlado(error));
    }

    const { status, data } = error.response;

    switch (status) {
      case 400:
        console.log('Error 400:', data);
        notifica.show(
          'Información no legible. Contacte con el administrador',
          {
            severity: 'error',
            autoHideDuration: 5000,
          });

        break;

      case 401:
        await dialogo.alert(
          'La sesión ha caducado y la aplicación se cerrará. ' +
          'Si desea continuar, vuelva a introducir sus credenciales');

        if (usrLogeado) {
          usrLogeado.setUsrLogeado(null);
        }

        // Forzamos a eliminar caches. Liberamos memoria
        window.location.replace('/');

        break;

      case 500:
        const msg_error_interno =
          'Error interno. Contacte con el administrador';

        if (typeof data === 'string' && data.startsWith('@@:')) {
          const mensajeUsuario = data.substring(3);
          notifica.show(mensajeUsuario,
            {
              severity: 'error',
              autoHideDuration: 10000,
            });
        } else {
          console.log('Error 500 interno:', data);
          notifica.show(msg_error_interno,
            {
              severity: 'error',
              autoHideDuration: 5000,
            });
        }

        break;
      default:
        logError('interceptor:', error);

        notifica.show(
          'Error inesperado. Contacte con el administrador',
          {
            severity: 'error',
            autoHideDuration: 5000,
          });
    }

    return Promise.reject(new NetErrorControlado(error));
  }
);

export { axios };