import { AxiosInstance } from "axios";
import { Incidencia } from "../modelos/incidencias";
import { instanceToPlain } from "class-transformer";

export interface IncidenciaApi {
  incidencias(inc: Incidencia): Promise<void>;
}


// Implementación de IncidenciaApi en modo producción
export class IncidenciaAxiosApi implements IncidenciaApi {
  private axios: AxiosInstance;

  constructor(axiosInstance: AxiosInstance) {
    this.axios = axiosInstance;
  }

  async incidencias(inc: Incidencia): Promise<void> {
    return this.axios.post('api/incidencias', instanceToPlain(inc));
  }
}

// Implementación de MarcajeApi en modo test
export class IncidenciaTestApi implements IncidenciaApi {
  async incidencias(_: Incidencia): Promise<void> {
    return;
  }
}