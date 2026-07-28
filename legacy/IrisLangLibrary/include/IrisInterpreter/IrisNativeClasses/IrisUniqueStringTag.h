#ifndef _H_IRISUNIQUESTRINGTAG_
#define _H_IRISUNIQUESTRINGTAG_

#include "IrisDevHeader.h"
#include <string>
using namespace std;

#if IR_USE_MEM_POOL
class IrisUniqueStringTag : public IrisObjectMemoryPoolInterface<IrisUniqueStringTag, POOLID_IrisUniqueStringTag>
#else
class IrisUniqueStringTag
#endif
{
private:
	wstring m_strWString;
	string m_strString;

public:

	const string& GetString();

	IrisUniqueStringTag(const string& strString);
	~IrisUniqueStringTag();

	string IrisUniqueStringTag::ws2s(const std::wstring& ws);
	wstring IrisUniqueStringTag::s2ws(const std::string& s);
};

#endif