import { AxiosRequestConfig } from 'axios';
import axios, { AxiosError } from 'axios';

import { UseNotifications } from '../hooks/useNotifications/useNotifications';
import { DialogHook } from '../hooks/useDialogs/useDialogs';
import { UseUsuarioLogeado } from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { logError } from '../error';
import { URL_BASE_API, URL_BASE_ROUTER } from '../config';

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

axios.defaults.baseURL = URL_BASE_API;
axios.defaults.withCredentials = true; // Incluye cookies HttpOnly

// Error controlado
//
// Es un error que se maneja en el interceptor
export class NetErrorControlado {
  constructor(public readonly origen: AxiosError) {
  }
}

export interface ConfigRequest extends AxiosRequestConfig {
  manejarErrorInesperado?: boolean;
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
        notifica.show(
          'Información no legible. Contacte con el administrador',
          {
            severity: 'error',
            autoHideDuration: 5000,
          });

        await logError('Error 400', dialogo?.alert, error);
        break;

      case 401:
        await dialogo.alert(
          'La sesión ha caducado y la aplicación se cerrará. ' +
          'Si desea continuar, vuelva a introducir sus credenciales');

        if (usrLogeado) {
          usrLogeado.setUsrLogeado(null);
        }

        // Forzamos a eliminar caches. Liberamos memoria
        window.location.replace(URL_BASE_ROUTER);

        break;

      case 500:
        const msg_error_interno =
          'Error interno. Contacte con el administrador';

        if (typeof data === 'string' && data.startsWith('@@:')) {
          let mensajeUsuario = data.substring(3).trim();
          let duracion = 15000;

          if (mensajeUsuario === "") {
            mensajeUsuario = msg_error_interno
            duracion = 5000
          }

          notifica.show(mensajeUsuario,
            {
              severity: 'error',
              autoHideDuration: duracion,
            });
        } else {
          notifica.show(msg_error_interno,
            {
              severity: 'error',
              autoHideDuration: 5000,
            });
          await logError('Error 500', dialogo?.alert, error);
        }

        break;
      default:
        if ((error.config as ConfigRequest)?.manejarErrorInesperado === true) {
          return Promise.reject(error);
        }

        notifica.show('Error inesperado. Contacte con el administrador', {
          severity: 'error',
          autoHideDuration: 5000,
        });
        await logError('Error desconocido', dialogo?.alert, error);
    }

    return Promise.reject(new NetErrorControlado(error));
  }
);

export { axios };