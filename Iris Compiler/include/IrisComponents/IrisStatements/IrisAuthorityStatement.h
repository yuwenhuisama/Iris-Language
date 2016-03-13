#ifndef _H_IRISAUTHORITYSTATEMENT_
#define _H_IRISAUTHORITYSTATEMENT_

#include "IrisStatement.h"
#include "IrisComponents/IrisComponentsDefines.h"

class IrisIdentifier;
class IrisAuthorityStatement :
	public IrisStatement
{
protected:
	IrisIdentifier* m_pMethodName = nullptr;
	IrisAuthorityEnvironment m_eEnvironment = IrisAuthorityEnvironment::Class;
	IrisAuthorityTarget m_eTarget = IrisAuthorityTarget::InstanceMethod;
	IrisAuthorityType m_eType = IrisAuthorityType::EveryOne;

public:

	virtual bool Generate();

	IrisAuthorityStatement(IrisAuthorityEnvironment eEnv, IrisAuthorityTarget eTar, IrisAuthorityType eType, IrisIdentifier* pMethodName);
	virtual ~IrisAuthorityStatement();
};

#endif