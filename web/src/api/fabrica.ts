import axios from "axios";
import { UsuariosApi, UsuariosAxiosApi, UsuariosTestApi } from "./usuarios";
import { MarcajeApi, MarcajeAxiosApi, MarcajeTestApi } from "./marcaje";
import { IncidenciaApi, IncidenciaAxiosApi, IncidenciaTestApi } from "./incidencias";

export class ContextoApi {
  constructor(
    public usuarios: UsuariosApi,
    public marcajes: MarcajeApi,
    public inc: IncidenciaApi) {
  }
}

// Variable global para el api (singleton)
let _api: ContextoApi | null = null;

export function api(): ContextoApi {
  if (_api == null) {
    throw Error('No se ha inicializado el API');
  }

  return _api;
}

// Crea el API de acceso a los servicios y lo inicializa
export function crearAPI(modoTest: boolean = false) {
  let usuarioApi: UsuariosApi;
  let marcajeApi: MarcajeApi;
  let incApi: IncidenciaApi;

  if (modoTest) {
    usuarioApi = new UsuariosTestApi();
    marcajeApi = new MarcajeTestApi();
    incApi = new IncidenciaTestApi();
  } else {
    usuarioApi = new UsuariosAxiosApi(axios);
    marcajeApi = new MarcajeAxiosApi(axios);
    incApi = new IncidenciaAxiosApi(axios);
  }

  const contexto = new ContextoApi(usuarioApi, marcajeApi, incApi);
  _api = contexto;
}
