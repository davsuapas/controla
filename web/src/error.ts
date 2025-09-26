// Escribe en la consola el error producido
export function logError(msg: string, error: any) {
  console.error(msg + ':', JSON.stringify(error, null, 2));
}