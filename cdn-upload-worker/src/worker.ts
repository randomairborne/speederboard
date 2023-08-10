export interface Env {
	DEAR_GOD: R2Bucket;
	// shasum adds a newline by default, if the output is wrong that is the problem
	// can use echo -n "input" | shasum -a 256
	SECRET_UPLOAD_KEY_SHA256: string;
}

export default {
	async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
		const { method, headers, body, url } = request;
		const path = new URL(url).pathname.replace(/^\//, '');
		const bearer_authorization = headers.get('authorization');
		const authorization = bearer_authorization?.replace('Bearer ', '');
		let maybeContentType = headers.get('content-type');
		let contentType: string | undefined;
		if (maybeContentType === null) {
			contentType = undefined;
		} else {
			contentType = maybeContentType;
		}
		if (authorization == null) {
			return new Response('authorization header required', { status: 400 });
		}
		const hashbuf = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(authorization));
		const hashArray = Array.from(new Uint8Array(hashbuf));
		const hash = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
		if (hash !== env.SECRET_UPLOAD_KEY_SHA256) {
			console.warn(`got auth hash ${hash}, wanted ${env.SECRET_UPLOAD_KEY_SHA256}`);
			return new Response('invalid auth', { status: 403 });
		}
		switch (method) {
			case 'PUT': {
				await env.DEAR_GOD.put(path, body, { httpMetadata: { contentType } });
				break;
			};
			case 'DELETE': {
				await env.DEAR_GOD.delete(path);
				break;
			};
			default: {
				return new Response('405 meth not allowed', { status: 405 });
			};
		}

		return new Response(null, { status: 204 });
	},
};
