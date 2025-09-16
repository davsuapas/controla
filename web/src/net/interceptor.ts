import { AxiosRequestConfig } from 'axios';
import axios, { AxiosError } from 'axios';

import { UseNotifications } from '../hooks/useNotifications/useNotifications';
import { DialogHook } from '../hooks/useDialogs/useDialogs';

// Variables globales para l interceptor
let dialogo: DialogHook | null = null;
let notifica: UseNotifications | null = null;
let redirigirALogin: (() => void) | null = null;

// Conecta el UI con el interceptor
export const configurarUI = (
  dialogoParam: DialogHook,
  notificaParam: UseNotifications,
  redireccionFn: () => void
) => {
  dialogo = dialogoParam;
  notifica = notificaParam;
  redirigirALogin = redireccionFn;
};

axios.defaults.timeout = 10000; //10 sg

const protocol = window.location.protocol;
const currentDomain = window.location.hostname;
axios.defaults.baseURL = `${protocol}//${currentDomain}:8080`;

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
    if (!error.response) {
      // Error de red
      if (notifica) {
        await notifica.show(
          'Error de conexión. Verifique su conexión a internet.');

        return Promise.reject(new NetErrorControlado(error));
      }
      return Promise.reject(error);
    }

    const { status, data } = error.response;
    const manejarAuth = (error.config as any)?.manejarAuth;

    let controlado = false;

    switch (status) {
      case 400:
        console.log('Error 400:', data);
        if (notifica) {
          notifica.show(
            'Información no legible. Contacte con el administrador',
            {
              severity: 'error',
              autoHideDuration: 5000,
            });
          controlado = true;
        }
        break;

      case 401:
        if (!manejarAuth) {
          if (dialogo) {
            await dialogo.alert(
              'La sesión ha caducado y la aplicación se cerrará. ' +
              'Si desea continuar, vuelva a introducir sus credenciales');
            controlado = true;
          }

          if (redirigirALogin) {
            redirigirALogin();
          }
        }
        break;

      case 500:
        const msg_error_interno =
          'Error interno. Contacte con el administrador';

        if (typeof data === 'string' && data.startsWith('@@:')) {
          const mensajeUsuario = data.substring(3);
          if (notifica) {
            notifica.show(mensajeUsuario,
              {
                severity: 'error',
                autoHideDuration: 10000,
              });
            controlado = true;
          }
        } else if (!data) {
          if (notifica) {
            notifica.show(msg_error_interno,
              {
                severity: 'error',
                autoHideDuration: 5000,
              });
            controlado = true;
          }
        } else {
          console.log('Error 500 interno:', data);
          if (notifica) {
            notifica.show(msg_error_interno,
              {
                severity: 'error',
                autoHideDuration: 5000,
              });
            controlado = true;
          }
        }
        break;
      default:
        console.log('Error ${status}:', data);
        if (notifica) {
          notifica.show(
            'Error inesperado. Contacte con el administrador',
            {
              severity: 'error',
              autoHideDuration: 5000,
            });
          controlado = true;
        }
        break;
    }

    if (controlado) {
      return Promise.reject(new NetErrorControlado(error));
    }

    return Promise.reject(error);
  }
);

export { axios };