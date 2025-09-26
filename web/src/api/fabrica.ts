import axios from "axios";
import { UsuariosApi, UsuariosAxiosApi, UsuariosTestApi } from "./usuarios";
import { RegistroApi, RegistroAxiosApi, RegistroTestApi } from "./registro";

export class ContextoApi {
  constructor(
    public usuarios: UsuariosApi,
    public registros: RegistroApi) {
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
  let registroApi: RegistroApi;

  if (modoTest) {
    usuarioApi = new UsuariosTestApi();
    registroApi = new RegistroTestApi();
  } else {
    usuarioApi = new UsuariosAxiosApi(axios);
    registroApi = new RegistroAxiosApi(axios);
  }

  const contexto = new ContextoApi(usuarioApi, registroApi);
  _api = contexto;
}
