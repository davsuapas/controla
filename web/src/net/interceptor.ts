import axios, { AxiosError } from 'axios';

// Variables globales para UI
let dialogo: any = null;
let redirigirALogin: (() => void) | null = null;

export const configurarUI = (
  dialogoFn: any,
  redireccionFn: () => void
) => {
  dialogo = dialogoFn;
  redirigirALogin = redireccionFn;
};

axios.defaults.timeout = 10000; //10 sg

import { AxiosRequestConfig } from 'axios';

export interface ConfiguracionPeticion extends AxiosRequestConfig {
  manejarAuth?: boolean;
}

// Interceptor de response para manejo de errores
axios.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    if (!error.response) {
      // Error de red
      if (dialogo) {
        await dialogo.alerta(
          "Error de conexión. Verifique su conexión a internet.");
      }
      return Promise.reject(error);
    }

    const { status, data } = error.response;
    const manejarAuth = (error.config as any)?.manejarAuth;

    switch (status) {
      case 400:
        console.error('Error 400:', data);
        if (dialogo) {
          await dialogo.alerta(
            "Información no legible. Contacte con el administrador");
        }
        break;

      case 401:
        if (!manejarAuth) {
          if (dialogo) {
            await dialogo.alerta(
              "La sesión ha caducado y la aplicación se cerrará. " +
              "Si desea continuar, vuelva a introducir sus credenciales");
          }
          if (redirigirALogin) {
            redirigirALogin();
          }
        }
        break;

      case 500:
        const msg_error_interno =
          "Error interno. Contacte con el administrador";

        if (typeof data === "string" && data.startsWith("@@:")) {
          const mensajeUsuario = data.substring(3);
          if (dialogo) {
            await dialogo.alerta(mensajeUsuario);
          }
        } else if (!data) {
          if (dialogo) {
            await dialogo.alerta(msg_error_interno);
          }
        } else {
          console.error("Error 500 interno:", data);
          if (dialogo) {
            await dialogo.alerta(msg_error_interno);
          }
        }
        break;

      default:
        console.error("Error ${status}:", data);
        if (dialogo) {
          await dialogo.alerta(
            "Error inesperado. Contacte con el administrador");
        }
        break;
    }

    return Promise.reject(error);
  }
);

export { axios };