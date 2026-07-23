import { Expose } from 'class-transformer';


export interface Localizacion {
  lat: number;
  lng: number;
  accuracy: number;
}

export class Config {
  localizacion: Localizacion | null;
  @Expose({ name: 'margen_recinto' })
  margenRecinto: number | null;

  constructor(data: Partial<Config>) {
    Object.assign(this, data);
  }

  static fromRequest(obj: any): Config {
    return new Config({
      localizacion: obj.localizacion ? {
        lat: Number(obj.localizacion.lat),
        lng: Number(obj.localizacion.lng),
        accuracy: Number(obj.localizacion.accuracy),
      } : null,
      margenRecinto: obj.margen_recinto ? Number(obj.margen_recinto) : null,
    });
  }

  static fromRequestArray(objs: any[]): Config[] {
    return objs.map(obj => Config.fromRequest(obj));
  }

  toServer(): {} {
    return {
      localizacion: this.localizacion ?? null,
      margen_recinto: this.margenRecinto ?? null,
    };
  }
}
