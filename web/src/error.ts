import dayjs from "dayjs";

// Escribe en la consola el error producido
export function logError(msg: string, error: any) {
  console.error(msg + ':', JSON.stringify(error, null, 2));
}

export function validarFechaHora(fechaHora: dayjs.Dayjs | null | undefined) {
  return fechaHora && fechaHora.isValid()
}
