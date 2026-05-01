import dayjs from "dayjs";
import { AxiosError } from 'axios'; // Importar AxiosError para la verificación de tipo
import { IS_DEBUG } from "./config";

/**
 * Registra un error en la consola y, si el modo depuración está activo, muestra una alerta técnica.
 * @param msg Mensaje descriptivo opcional que se antepondrá al error.
 * @param alert Objeto o función de alerta (ej. dialogo.alert).
 * @param error El error producido (AxiosError, Error, string, etc).
 */
export async function logError(msg: string | undefined, alert: any, error: any) {
  const reportText = formatErrorForDebug(error);
  const fullMsg = msg ? `${msg}\n\n${reportText}` : reportText;

  if (IS_DEBUG && alert && typeof alert === 'function') {
    await alert(fullMsg);
  } else {
    // En consola mostramos el objeto real para inspección si está disponible
    if (msg) {
      console.error(msg, error);
    } else {
      console.error(error);
    }
  }
}

/**
 * Helper interno para formatear el detalle técnico.
 */
function formatErrorForDebug(error: any): string {
  const report: { [key: string]: any } = {
    timestamp: dayjs().format('YYYY-MM-DD HH:mm:ss'),
    appId: import.meta.env.VITE_CONTROLA_WEB_APP || 'unknown',
    currentUrl: window.location.href,
  };

  if (error instanceof AxiosError) {
    report.type = 'AxiosError';
    report.message = error.message;
    report.code = error.code;
    report.isAxiosError = error.isAxiosError;
    if (error.response) {
      report.response = {
        status: error.response.status,
        statusText: error.response.statusText,
        data: error.response.data,
      };
    }
    if (error.config) {
      report.requestConfig = {
        method: error.config.method,
        url: error.config.url,
      };
    }
  } else if (error instanceof Error) {
    report.type = 'JavaScriptError';
    report.name = error.name;
    report.message = error.message;
    report.stack = error.stack;
  } else if (typeof error === 'string') {
    report.type = 'StringError';
    report.message = error;
  } else if (typeof error === 'object' && error !== null) {
    report.type = 'GenericObjectError';
    try {
      report.details = JSON.parse(JSON.stringify(error)); // Intenta copiar profundamente para evitar referencias circulares
    } catch (e) {
      report.details = String(error); // Fallback a conversión a string
    }
  } else {
    report.type = 'UnknownError';
    report.details = String(error);
  }

  return `--- DEBUG ERROR REPORT ---\n${JSON.stringify(report, null, 2)}\n-------------------------`;
}

export function validarFechaHora(fechaHora: dayjs.Dayjs | null | undefined) {
  return fechaHora && fechaHora.isValid()
}
