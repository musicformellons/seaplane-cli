import { jest, expect } from '@jest/globals';

import seaFetch from '../../src/api/seaFetch';
jest.mock('../../src/api/seaFetch', () => jest.fn());

type Body = Record<string, unknown> | string;
const textBody = (body: Body) =>
  Promise.resolve({
    ok: () => true,
    text: () => Promise.resolve(body),
  });
export const postTokenMock = {
  post: (url: string) => {
    expect(url).toBe('https://flightdeck.cplane.cloud/v1/token');

    return textBody({ token: 'test_token' });
  },
};

export const mockIdentify = () => {
  seaFetch.mockImplementation(() => ({
    post: () => {
      return Promise.resolve({
        ok: () => true,
        text: () => Promise.resolve({ token: 'test_token' }),
      });
    },
  }));
};

export const mockServer = (serverUrl: string, auth = true) => ({
  get: (path: string, body: Body) => {
    const authPost = auth ? postTokenMock : {};
    seaFetch.mockImplementation(() => ({
      ...authPost,
      get: (url: string) => {
        expect(url).toBe(serverUrl + path);

        return textBody(body);
      },
    }));
  },
  delete: (path: string, body: Body) => {
    seaFetch.mockImplementation(() => ({
      ...postTokenMock,
      delete: (url: string) => {
        expect(url).toBe(serverUrl + path);

        return textBody(body);
      },
    }));
  },
  put: (path: string, body: string) => {
    seaFetch.mockImplementation(() => ({
      ...postTokenMock,
      put: (url: string) => {
        expect(url).toBe(serverUrl + path);

        return textBody(body);
      },
    }));
  },
  post: (path: string, body: Body, returnBody: Body) => {
    if (auth) {
      seaFetch.mockReturnValue(postTokenMock).mockReturnValueOnce({
        post: (url: string, postBody: string) => {
          expect(url).toBe(serverUrl + path);
          expect(postBody).toBe(JSON.stringify(body));

          return textBody(returnBody);
        },
      });
    } else {
      seaFetch.mockImplementation(() => ({
        post: (url: string, postBody: string) => {
          expect(url).toBe(serverUrl + path);
          expect(postBody).toBe(JSON.stringify(body));

          return textBody(returnBody);
        },
      }));
    }
  },
});
