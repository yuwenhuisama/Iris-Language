#ifndef _H_IRISINTERSTRING_
#define _H_IRISINTERSTRING_

#include "IrisInterfaces/IIrisInterString.h"

#include <string>
using namespace std;

class IrisInternString : public IIrisInterString
{
public:
	struct IrisInerStringHash {
		size_t operator () (const IrisInternString& isString) const {
			size_t nHash = 0;
			if (!isString.IsHashed()) {
				const string& str = isString.GetSTLString();
				size_t h = 0;
				for (size_t i = 0;i < str.length();++i) {
					h = (h << 5) - h + str[i];
				}
				const_cast<IrisInternString&>(isString).SetHashed(true);
				const_cast<IrisInternString&>(isString).SetHashValue(h);
				nHash = h;
			}
			else {
				nHash = isString.GetHashValue();
			}
			return nHash;
		}
	};

private:
	string m_strString = "";
	size_t m_nHash = 0;
	bool m_bHashed = false;

public:
	IrisInternString(const string& strInterString);
	IrisInternString(const char* szInternString);

	const char* GetCTypeString() const;
	const string& GetSTLString() const;
	
	bool operator == (const IrisInternString& iisString) const {
		return m_strString == iisString.m_strString;
	}

	bool operator != (const IrisInternString& iisString) const {
		return !(*this == iisString);
	}

	void SetHashed(bool bFlag) ;
	bool IsHashed() const;
	void SetHashValue(size_t nHashValue);
	size_t GetHashValue() const;

	~IrisInternString();
};
#endif