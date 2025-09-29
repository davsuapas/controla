import axios from "axios";
import { UsuariosApi, UsuariosAxiosApi, UsuariosTestApi } from "./usuarios";
import { MarcajeApi, MarcajeAxiosApi, MarcajeTestApi } from "./marcaje";

export class ContextoApi {
  constructor(
    public usuarios: UsuariosApi,
    public marcajes: MarcajeApi) {
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

  if (modoTest) {
    usuarioApi = new UsuariosTestApi();
    marcajeApi = new MarcajeTestApi();
  } else {
    usuarioApi = new UsuariosAxiosApi(axios);
    marcajeApi = new MarcajeAxiosApi(axios);
  }

  const contexto = new ContextoApi(usuarioApi, marcajeApi);
  _api = contexto;
}
