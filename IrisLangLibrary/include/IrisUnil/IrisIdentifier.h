#ifndef _H_IRISIDENTIFIER_
#define _H_IRISIDENTIFIER_

#include "IrisComponents/IrisComponentsDefines.h"
#include <string>
using namespace std;

class IrisIdentifier
{
private:
	string m_strIdentifier;
	IrisIdentifilerType m_eType = IrisIdentifilerType::Constance;

public:
	IrisIdentifier(IrisIdentifilerType eType, char* szIdentifier);
	~IrisIdentifier();

	inline IrisIdentifilerType GetType() { return m_eType; }
	const string& GetIdentifierString() { return m_strIdentifier; }

	//friend class IrisMethod;
	//friend class IrisExpression;
	//friend class IrisStatement;
	//friend class IrisClosureBlock;
};

#endif