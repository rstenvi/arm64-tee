#include <stddef.h>
#include <stdint.h>

void* memcpy(void* dest, const void* src, size_t n)	{
	size_t i;
	for(i = 0; i < n; i++)	{
		*((uint8_t*)(dest + i)) = *((uint8_t*)(src + i));
	}
	return dest;
}
void* memmove(void* dest, const void* src, size_t n)	{
	// These are equal in our case
	memcpy(dest, src, n);
}
int memcmp(const void* s1, const void* s2, size_t n)	{
	size_t i;
	uint8_t* _s1, * _s2;
	for(i = 0; i < n; i++)	{
		_s1 = ((uint8_t*)(s1 + i));
		_s2 = ((uint8_t*)(s2 + i));
		if(*_s1 != *_s2)	return *_s1 - *_s2;
	}
	return 0;
}
void *memset(void *s, int c, size_t n)	{
	uint8_t* _s = (uint8_t*)s;
	size_t i;
	for(i = 0; i < n; i++)	_s[i] = c;
	return s;
}
