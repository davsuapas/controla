import { AxiosInstance } from 'axios';
import { Config } from '../modelos/config';

export interface ConfigApi {
  data(): Promise<Config>;
  actualizar(config: Config): Promise<void>;
}

export class ConfigAxiosApi implements ConfigApi {
  private axios: AxiosInstance;

  constructor(axiosInstance: AxiosInstance) {
    this.axios = axiosInstance;
  }

  async data(): Promise<Config> {
    const response = await this.axios.get('config');
    return Config.fromRequest(response.data);
  }

  async actualizar(config: Config): Promise<void> {
    await this.axios.put('config', config.toServer());
  }
}

export class ConfigTestApi implements ConfigApi {
  async data(): Promise<Config> {
    return Promise.resolve(Config.fromRequest({
      localizacion: {
        lat: 40.4168,
        lng: -3.7038,
        accuracy: 15,
      },
    }));
  }

  async actualizar(_: Config): Promise<void> {
    return;
  }
}
