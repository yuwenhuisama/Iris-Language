#ifndef _H_IRISINTERSTRING_
#define _H_IRISINTERSTRING_

#include "IrisInterfaces/IIrisInterString.h"

#include <string>
using namespace std;

class IrisInternString : public IIrisInterString
{
private:
	string m_strString = "";
	size_t m_nHash = 0;

public:
	IrisInternString(const string& strInterString);
	IrisInternString(const char* szInterString);

	inline const char* GetCTypeString() const { return m_strString.c_str(); }

	~IrisInternString();
};

#endif