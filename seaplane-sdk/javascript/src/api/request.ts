import Identify from './identify';

import { log } from '../logging';
import { HTTPError } from '../model/errors';

export default class Request {
  private identify: Identify;

  constructor(identify: Identify) {
    this.identify = identify;
  }

  private async renewIfFails(error: any, bodyError: string, request: (token: string) => Promise<any>): Promise<any> {
    const httpError = new HTTPError(error.status, bodyError);

    if (error.status != 401 || !this.identify.autoRenew) {
      throw httpError;
    }

    log.info('Auto-Renew, renewing the token...');
    const token = await this.identify.renewToken();
    return await request(token);
  }

  async send(request: (token: string) => Promise<any>): Promise<any> {
    // eslint-disable-line

    const accessToken: string = this.identify.accessToken || (await this.identify.getToken());

    const response = await request(accessToken);
    const bodyText = await response.text();

    if (response.ok) {
      return this.parse(bodyText) ?? bodyText;
    } else {
      log.error(`Request error: ${bodyText}`);
      return await this.renewIfFails(response, bodyText, request);
    }
  }

  private parse(body: string): any | undefined {
    // eslint-disable-line
    try {
      return JSON.parse(body);
    } catch (e) {
      return undefined;
    }
  }
}
